use filters;
use input::TemplateInput;
use parser::{self, Cond, Expr, Macro, MatchParameter, MatchVariant, Node, Target, When, WS};
use path;

use quote::{Tokens, ToTokens};

use std::{cmp, hash, str};
use std::path::Path;
use std::collections::{HashMap, HashSet};

use syn;


pub fn generate(input: &TemplateInput, nodes: &[Node], imported: &HashMap<(&str, &str), Macro>) -> String {
    Generator::default().build(&State::new(input, nodes, imported))
}

struct State<'a> {
    input: &'a TemplateInput<'a>,
    nodes: &'a [Node<'a>],
    blocks: Vec<&'a Node<'a>>,
    macros: MacroMap<'a>,
    trait_name: String,
    derived: bool,
}

impl<'a> State<'a> {
    fn new<'n>(input: &'n TemplateInput, nodes: &'n [Node], imported: &'n HashMap<(&'n str, &'n str), Macro<'n>>) -> State<'n> {
        let mut base: Option<&Expr> = None;
        let mut blocks = Vec::new();
        let mut macros = HashMap::new();
        for n in nodes.iter() {
            match *n {
                Node::Extends(ref path) => {
                    match base {
                        Some(_) => panic!("multiple extend blocks found"),
                        None => { base = Some(path); },
                    }
                },
                ref def @ Node::BlockDef(_, _, _, _) => {
                    blocks.push(def);
                },
                Node::Macro(name, ref m) => {
                    macros.insert((None, name), m);
                },
                _ => {},
            }
        }
        for (&(scope, name), ref m) in imported {
            macros.insert((Some(scope), name), m);
        }
        State {
            input,
            nodes,
            blocks,
            macros,
            trait_name: trait_name_for_path(&base, &input.path),
            derived: base.is_some(),
        }
    }
}

fn trait_name_for_path(base: &Option<&Expr>, path: &Path) -> String {
    let rooted_path = match *base {
        Some(&Expr::StrLit(user_path)) => {
            path::find_template_from_path(user_path, Some(path))
        },
        _ => path.to_path_buf(),
    };

    let mut res = String::new();
    res.push_str("TraitFrom");
    for c in rooted_path.to_string_lossy().chars() {
        if c.is_alphanumeric() {
            res.push(c);
        } else {
            res.push_str(&format!("{:x}", c as u32));
        }
    }
    res
}

fn get_parent_type(ast: &syn::DeriveInput) -> Option<&syn::Ty> {
    match ast.body {
        syn::Body::Struct(ref data) => {
            data.fields().iter().filter_map(|f| {
                f.ident.as_ref().and_then(|name| {
                    if name.as_ref() == "_parent" {
                        Some(&f.ty)
                    } else {
                        None
                    }
                })
            })
        },
        _ => panic!("derive(Template) only works for struct items"),
    }.next()
}

struct Generator<'a> {
    buf: String,
    indent: u8,
    start: bool,
    locals: SetChain<'a, &'a str>,
    next_ws: Option<&'a str>,
    skip_ws: bool,
}

impl<'a> Generator<'a> {

    fn new<'n>(locals: SetChain<'n, &'n str>, indent: u8) -> Generator<'n> {
        Generator {
            buf: String::new(),
            indent: indent,
            start: true,
            locals: locals,
            next_ws: None,
            skip_ws: false,
        }
    }

    fn default<'n>() -> Generator<'n> {
        Self::new(SetChain::new(), 0)
    }

    fn child(&mut self) -> Generator {
        let locals = SetChain::with_parent(&self.locals);
        Self::new(locals, self.indent)
    }

    // Takes a State and generates the relevant implementations.
    fn build(mut self, state: &'a State) -> String {
        if !state.blocks.is_empty() {
            if !state.derived {
                self.define_trait(state);
            } else {
                let parent_type = get_parent_type(state.input.ast)
                    .expect("expected field '_parent' in extending template struct");
                self.deref_to_parent(state, parent_type);
            }

            let trait_nodes = if !state.derived { Some(&state.nodes[..]) } else { None };
            self.impl_trait(state, trait_nodes);
            self.impl_template_for_trait(state);
        } else {
            self.impl_template(state);
        }
        self.impl_display(state);
        if cfg!(feature = "iron") {
            self.impl_modifier_response(state);
        }
        if cfg!(feature = "rocket") {
            self.impl_responder(state);
        }
        self.buf
    }

    // Implement `Template` for the given context struct.
    fn impl_template(&mut self, state: &'a State) {
        self.write_header(state, "::askama::Template", &[]);
        self.writeln("fn render_into(&self, writer: &mut ::std::fmt::Write) -> \
                      ::askama::Result<()> {");
        self.writeln("#[allow(unused_imports)] use ::std::ops::Deref as HiddenDerefTrait;");
        self.handle(state, state.nodes, AstLevel::Top);
        self.flush_ws(&WS(false, false));
        self.writeln("Ok(())");
        self.writeln("}");
        self.writeln("}");
    }

    // Implement `Display` for the given context struct.
    fn impl_display(&mut self, state: &'a State) {
        self.write_header(state, "::std::fmt::Display", &[]);
        self.writeln("fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {");
        self.writeln("self.render_into(f).map_err(|_| ::std::fmt::Error {})");
        self.writeln("}");
        self.writeln("}");
    }

    // Implement `Deref<Parent>` for an inheriting context struct.
    fn deref_to_parent(&mut self, state: &'a State, parent_type: &syn::Ty) {
        self.write_header(state, "::std::ops::Deref", &[]);
        let mut tokens = Tokens::new();
        parent_type.to_tokens(&mut tokens);
        self.writeln(&format!("type Target = {};", tokens.as_str()));
        self.writeln("fn deref(&self) -> &Self::Target {");
        self.writeln("&self._parent");
        self.writeln("}");
        self.writeln("}");
    }

    // Implement `TraitFromPathName` for the given context struct.
    fn impl_trait(&mut self, state: &'a State, nodes: Option<&'a [Node]>) {
        self.write_header(state, &state.trait_name, &[]);
        self.write_block_defs(state);

        self.writeln("#[allow(unused_variables)]");
        self.writeln(&format!(
            "fn render_trait_into(&self, timpl: &{}, writer: &mut ::std::fmt::Write) \
             -> ::askama::Result<()> {{",
            state.trait_name));
        self.writeln("#[allow(unused_imports)] use ::std::ops::Deref as HiddenDerefTrait;");

        if let Some(nodes) = nodes {
            self.handle(state, nodes, AstLevel::Top);
            self.flush_ws(&WS(false, false));
        } else {
            self.writeln("self._parent.render_trait_into(self, writer)?;");
        }

        self.writeln("Ok(())");
        self.writeln("}");
        self.flush_ws(&WS(false, false));
        self.writeln("}");
    }

    // Implement `Template` for templates that implement a template trait.
    fn impl_template_for_trait(&mut self, state: &'a State) {
        self.write_header(state, "::askama::Template", &[]);
        self.writeln("fn render_into(&self, writer: &mut ::std::fmt::Write) \
                      -> ::askama::Result<()> {");
        if state.derived {
            self.writeln("self._parent.render_trait_into(self, writer)?;");
        } else {
            self.writeln("self.render_trait_into(self, writer)?;");
        }
        self.writeln("Ok(())");
        self.writeln("}");
        self.writeln("}");
    }

    // Defines the `TraitFromPathName` trait.
    fn define_trait(&mut self, state: &'a State) {
        self.writeln(&format!("trait {} {{", state.trait_name));
        self.write_block_defs(state);
        self.writeln(&format!(
            "fn render_trait_into(&self, timpl: &{}, writer: &mut ::std::fmt::Write) \
             -> ::askama::Result<()>;",
            state.trait_name));
        self.writeln("}");
    }

    // Implement iron's Modifier<Response> if enabled
    fn impl_modifier_response(&mut self, state: &'a State) {
        self.write_header(state, "::askama::iron::Modifier<::askama::iron::Response>", &[]);
        self.writeln("fn modify(self, res: &mut ::askama::iron::Response) {");
        self.writeln("res.body = Some(Box::new(self.render().unwrap().into_bytes()));");
        self.writeln("}");
        self.writeln("}");
    }

    // Implement Rocket's `Responder`.
    fn impl_responder(&mut self, state: &'a State) {
        self.write_header(state, "::askama::rocket::Responder<'r>", &["'r"]);
        self.writeln("fn respond_to(self, _: &::askama::rocket::Request) \
                      -> ::askama::rocket::Result<'r> {");

        let ext = match state.input.path.extension() {
            Some(s) => s.to_str().unwrap(),
            None => "txt",
        };
        self.writeln(&format!("::askama::rocket::respond(&self, {:?})", ext));

        self.writeln("}");
        self.writeln("}");
    }

    // Writes header for the `impl` for `TraitFromPathName` or `Template`
    // for the given context struct.
    fn write_header(&mut self, state: &'a State, target: &str, extra_anno: &[&str]) {
        let mut full_anno = Tokens::new();
        let mut orig_anno = Tokens::new();
        let need_anno = !state.input.ast.generics.lifetimes.is_empty() ||
                        !state.input.ast.generics.ty_params.is_empty() ||
                        !extra_anno.is_empty();
        if need_anno {
            full_anno.append("<");
            orig_anno.append("<");
        }

        let (mut full_sep, mut orig_sep) = (false, false);
        for lt in &state.input.ast.generics.lifetimes {
            if full_sep {
                full_anno.append(",");
            }
            if orig_sep {
                orig_anno.append(",");
            }
            lt.to_tokens(&mut full_anno);
            lt.to_tokens(&mut orig_anno);
            full_sep = true;
            orig_sep = true;
        }

        for anno in extra_anno {
            if full_sep {
                full_anno.append(",");
            }
            full_anno.append(anno);
            full_sep = true;
        }

        for param in &state.input.ast.generics.ty_params {
            if full_sep {
                full_anno.append(",");
            }
            if orig_sep {
                orig_anno.append(",");
            }
            let mut impl_param = param.clone();
            impl_param.default = None;
            impl_param.to_tokens(&mut full_anno);
            param.ident.to_tokens(&mut orig_anno);
            full_sep = true;
            orig_sep = true;
        }

        if need_anno {
            full_anno.append(">");
            orig_anno.append(">");
        }

        let mut where_clause = Tokens::new();
        state.input.ast.generics.where_clause.to_tokens(&mut where_clause);
        self.writeln(&format!("impl{} {} for {}{}{} {{",
                              full_anno.as_str(), target, state.input.ast.ident.as_ref(),
                              orig_anno.as_str(), where_clause.as_str()));
    }

    /* Helper methods for handling node types */

    fn handle(&mut self, state: &'a State, nodes: &'a [Node], level: AstLevel) {
        for n in nodes {
            match *n {
                Node::Lit(lws, val, rws) => { self.write_lit(lws, val, rws); }
                Node::Comment() => {},
                Node::Expr(ref ws, ref val) => { self.write_expr(state, ws, val); },
                Node::LetDecl(ref ws, ref var) => { self.write_let_decl(ws, var); },
                Node::Let(ref ws, ref var, ref val) => { self.write_let(ws, var, val); },
                Node::Cond(ref conds, ref ws) => {
                    self.write_cond(state, conds, ws);
                },
                Node::Match(ref ws1, ref expr, ref inter, ref arms, ref ws2) => {
                    self.write_match(state, ws1, expr, inter, arms, ws2);
                },
                Node::Loop(ref ws1, ref var, ref iter, ref body, ref ws2) => {
                    self.write_loop(state, ws1, var, iter, body, ws2);
                },
                Node::BlockDef(ref ws1, name, _, ref ws2) => {
                    if let AstLevel::Nested = level {
                        panic!("blocks ('{}') are only allowed at the top level", name);
                    }
                    self.write_block(ws1, name, ws2);
                },
                Node::Include(ref ws, path) => {
                    self.handle_include(state, ws, path);
                },
                Node::Call(ref ws, scope, name, ref args) => {
                    self.write_call(state, ws, scope, name, args);
                },
                Node::Macro(_, ref m) => {
                    if let AstLevel::Nested = level {
                        panic!("macro blocks only allowed at the top level");
                    }
                    self.flush_ws(&m.ws1);
                    self.prepare_ws(&m.ws2);
                },
                Node::Import(ref ws, _, _) => {
                    if let AstLevel::Nested = level {
                        panic!("import blocks only allowed at the top level");
                    }
                    self.handle_ws(ws);
                },
                Node::Extends(_) => {
                    if let AstLevel::Nested = level {
                        panic!("extend blocks only allowed at the top level");
                    }
                    // No whitespace handling: child template top-level is not used,
                    // except for the blocks defined in it.
                },
            }
        }
    }

    fn write_block_defs(&mut self, state: &'a State) {
        for b in &state.blocks {
            if let Node::BlockDef(ref ws1, name, ref nodes, ref ws2) = **b {
                self.writeln("#[allow(unused_variables)]");
                self.writeln(&format!(
                    "fn render_block_{}_into(&self, writer: &mut ::std::fmt::Write) \
                     -> ::askama::Result<()> {{",
                    name));
                self.prepare_ws(ws1);

                self.locals.push();
                self.handle(state, nodes, AstLevel::Nested);
                self.locals.pop();

                self.flush_ws(ws2);
                self.writeln("Ok(())");
                self.writeln("}");
            } else {
                panic!("only block definitions allowed here");
            }
        }
    }

    fn write_cond(&mut self, state: &'a State, conds: &'a [Cond], ws: &WS) {
        for (i, &(ref cws, ref cond, ref nodes)) in conds.iter().enumerate() {
            self.handle_ws(cws);
            match *cond {
                Some(ref expr) => {
                    if i == 0 {
                        self.write("if ");
                    } else {
                        self.dedent();
                        self.write("} else if ");
                    }
                    self.visit_expr(expr);
                },
                None => {
                    self.dedent();
                    self.write("} else");
                },
            }
            self.writeln(" {");
            self.locals.push();
            self.handle(state, nodes, AstLevel::Nested);
            self.locals.pop();
        }
        self.handle_ws(ws);
        self.writeln("}");
    }

    fn write_match(&mut self, state: &'a State, ws1: &WS, expr: &Expr, inter: &'a str, arms:
                   &'a [When], ws2: &WS) {
        self.flush_ws(ws1);
        if !inter.is_empty() {
            self.next_ws = Some(inter);
        }

        self.write("match ");
        self.write("(&");
        self.visit_expr(expr);
        self.write(").deref()");
        self.writeln(" {");

        for arm in arms {
            let &(ref ws, ref variant, ref params, ref body) = arm;
            self.locals.push();
            match *variant {
                Some(ref param) => {
                    self.visit_match_variant(param);
                },
                None => self.write("_"),
            };
            if params.len() > 0 {
                self.write("(");
                for (i, param) in params.iter().enumerate() {
                    if let MatchParameter::Name(ref p) = *param {
                        self.locals.insert(p);
                    }
                    if i > 0 {
                        self.write(", ");
                    }
                    self.visit_match_param(param);
                }
                self.write(")");
            }
            self.writeln(" => {");
            self.handle_ws(ws);
            self.handle(state, body, AstLevel::Nested);
            self.writeln("}");
            self.locals.pop();
        }

        self.writeln("}");
        self.handle_ws(ws2);
    }

    fn write_loop(&mut self, state: &'a State, ws1: &WS, var: &'a Target, iter: &Expr,
                  body: &'a [Node], ws2: &WS) {
        self.handle_ws(ws1);
        self.locals.push();
        self.write("for (_loop_index, ");
        let targets = self.visit_target(var);
        for name in &targets {
            self.locals.insert(name);
            self.write(name);
        }
        self.write(") in (&");
        self.visit_expr(iter);
        self.writeln(").into_iter().enumerate() {");

        self.handle(state, body, AstLevel::Nested);
        self.handle_ws(ws2);
        self.writeln("}");
        self.locals.pop();
    }

    fn write_call(&mut self, state: &'a State, ws: &WS, scope: Option<&str>, name: &str,
                  args: &[Expr]) {
        let def = state.macros.get(&(scope, name)).unwrap_or_else(|| {
            if let Some(ref s) = scope {
                panic!(format!("macro '{}::{}' not found", s, name));
            } else {
                panic!(format!("macro '{}' not found", name));
            }
        });

        self.flush_ws(ws); // Cannot handle_ws() here: whitespace from macro definition comes first
        self.locals.push();
        self.writeln("{");
        self.prepare_ws(&def.ws1);

        for (i, arg) in def.args.iter().enumerate() {
            self.write(&format!("let {} = &", arg));
            self.visit_expr(args.get(i)
                .expect(&format!("macro '{}' takes more than {} arguments", name, i)));
            self.writeln(";");
            self.locals.insert(arg);
        }
        self.handle(state, &def.nodes, AstLevel::Nested);

        self.flush_ws(&def.ws2);
        self.writeln("}");
        self.locals.pop();
        self.prepare_ws(ws);
    }

    fn handle_include(&mut self, state: &'a State, ws: &WS, path: &str) {
        self.prepare_ws(ws);
        let path = path::find_template_from_path(path, Some(&state.input.path));
        let src = path::get_template_source(&path);
        let nodes = parser::parse(&src);
        let nested = {
            let mut gen = self.child();
            gen.handle(state, &nodes, AstLevel::Nested);
            gen.buf
        };
        self.buf.push_str(&nested);
        self.flush_ws(ws);
    }

    fn write_let_decl(&mut self, ws: &WS, var: &'a Target) {
        self.handle_ws(ws);
        self.write("let ");
        match *var {
            Target::Name(name) => {
                self.locals.insert(name);
                self.write(name);
            },
        }
        self.writeln(";");
    }

    fn write_let(&mut self, ws: &WS, var: &'a Target, val: &Expr) {
        self.handle_ws(ws);
        match *var {
            Target::Name(name) => {
                if !self.locals.contains(name) {
                    self.write("let ");
                    self.locals.insert(name);
                }
                self.write(name);
            },
        }
        self.write(" = ");
        self.visit_expr(val);
        self.writeln(";");
    }

    fn write_block(&mut self, ws1: &WS, name: &str, ws2: &WS) {
        self.flush_ws(ws1);
        self.writeln(&format!("timpl.render_block_{}_into(writer)?;", name));
        self.prepare_ws(ws2);
    }

    fn write_expr(&mut self, state: &'a State, ws: &WS, s: &Expr) {
        self.handle_ws(ws);
        self.write("let askama_expr = &");
        let wrapped = self.visit_expr(s);
        self.writeln(";");

        use self::DisplayWrap::*;
        use super::input::EscapeMode::*;
        self.write("writer.write_fmt(format_args!(\"{}\", ");
        self.write(match (wrapped, &state.input.meta.escaping) {
            (Wrapped, &Html) |
            (Wrapped, &None) |
            (Unwrapped, &None) => "askama_expr",
            (Unwrapped, &Html) => "&::askama::MarkupDisplay::from(askama_expr)",
        });
        self.writeln("))?;");
    }

    fn write_lit(&mut self, lws: &'a str, val: &str, rws: &'a str) {
        assert!(self.next_ws.is_none());
        if !lws.is_empty() {
            if self.skip_ws {
                self.skip_ws = false;
            } else if val.is_empty() {
                assert!(rws.is_empty());
                self.next_ws = Some(lws);
            } else {
                self.writeln(&format!("writer.write_str({:#?})?;",
                                      lws));
            }
        }
        if !val.is_empty() {
            self.writeln(&format!("writer.write_str({:#?})?;", val));
        }
        if !rws.is_empty() {
            self.next_ws = Some(rws);
        }
    }

    /* Visitor methods for expression types */

    fn visit_expr(&mut self, expr: &Expr) -> DisplayWrap {
        match *expr {
            Expr::NumLit(s) => self.visit_num_lit(s),
            Expr::StrLit(s) => self.visit_str_lit(s),
            Expr::Var(s) => self.visit_var(s),
            Expr::Path(ref path) => self.visit_path(path),
            Expr::Array(ref elements) => self.visit_array(elements),
            Expr::Attr(ref obj, name) => self.visit_attr(obj, name),
            Expr::Filter(name, ref args) => self.visit_filter(name, args),
            Expr::BinOp(op, ref left, ref right) =>
                self.visit_binop(op, left, right),
            Expr::Group(ref inner) => self.visit_group(inner),
            Expr::MethodCall(ref obj, method, ref args) =>
                self.visit_method_call(obj, method, args),
        }
    }

    fn visit_match_variant(&mut self, param: &MatchVariant) -> DisplayWrap {
        match *param {
            MatchVariant::StrLit(s) => self.visit_str_lit(s),
            MatchVariant::NumLit(s) => {
                // Variants need to be references until match-modes land
                self.write("&");
                self.visit_num_lit(s)
            },
            MatchVariant::Name(s) => {
                self.write("&");
                self.write(s);
                DisplayWrap::Unwrapped
            }
            MatchVariant::Path(ref s) => {
                self.write("&");
                self.write(&s.join("::"));
                DisplayWrap::Unwrapped
            }
        }
    }

    fn visit_match_param(&mut self, param: &MatchParameter) -> DisplayWrap {
        match *param {
            MatchParameter::NumLit(s) => self.visit_num_lit(s),
            MatchParameter::StrLit(s) => self.visit_str_lit(s),
            MatchParameter::Name(s) => {
                self.write("ref ");
                self.write(s);
                DisplayWrap::Unwrapped
            }
        }
    }

    fn visit_filter(&mut self, name: &str, args: &[Expr]) -> DisplayWrap {
        if name == "format" {
            self._visit_format_filter(args);
            return DisplayWrap::Unwrapped;
        } else if name == "join" {
            self._visit_join_filter(args);
            return DisplayWrap::Unwrapped;
        }

        if filters::BUILT_IN_FILTERS.contains(&name) {
            self.write(&format!("::askama::filters::{}(&", name));
        } else {
            self.write(&format!("filters::{}(&", name));
        }

        self._visit_filter_args(args);
        self.write(")?");
        if name == "safe" || name == "escape" || name == "e" || name == "json" {
            DisplayWrap::Wrapped
        } else {
            DisplayWrap::Unwrapped
        }
    }

    fn _visit_format_filter(&mut self, args: &[Expr]) {
        self.write("format!(");
        self._visit_filter_args(args);
        self.write(")");
    }

    // Force type coercion on first argument to `join` filter (see #39).
    fn _visit_join_filter(&mut self, args: &[Expr]) {
        self.write("::askama::filters::join((&");
        for (i, arg) in args.iter().enumerate() {
            if i > 0 {
                self.write(", &");
            }
            self.visit_expr(arg);
            if i == 0 {
                self.write(").into_iter()");
            }
        }
        self.write(")?");
    }

    fn _visit_filter_args(&mut self, args: &[Expr]) {
        for (i, arg) in args.iter().enumerate() {
            if i > 0 {
                self.write(", &");
            }
            self.visit_expr(arg);
        }
    }

    fn visit_attr(&mut self, obj: &Expr, attr: &str) -> DisplayWrap {
        if let Expr::Var(name) = *obj {
            if name == "loop" {
                self.write("_loop_index");
                if attr == "index" {
                    self.write(" + 1");
                    return DisplayWrap::Unwrapped;
                } else if attr == "index0" {
                    return DisplayWrap::Unwrapped;
                } else {
                    panic!("unknown loop variable");
                }
            }
        }
        self.visit_expr(obj);
        self.write(&format!(".{}", attr));
        DisplayWrap::Unwrapped
    }

    fn visit_method_call(&mut self, obj: &Expr, method: &str, args: &[Expr]) -> DisplayWrap {
        self.visit_expr(obj);
        self.write(&format!(".{}(", method));
        for (i, arg) in args.iter().enumerate() {
            if i > 0 {
                self.write(", ");
            }
            self.visit_expr(arg);
        }
        self.write(")");
        DisplayWrap::Unwrapped
    }

    fn visit_binop(&mut self, op: &str, left: &Expr, right: &Expr) -> DisplayWrap {
        self.visit_expr(left);
        self.write(&format!(" {} ", op));
        self.visit_expr(right);
        DisplayWrap::Unwrapped
    }

    fn visit_group(&mut self, inner: &Expr) -> DisplayWrap {
        self.write("(");
        self.visit_expr(inner);
        self.write(")");
        DisplayWrap::Unwrapped
    }

    fn visit_array(&mut self, elements: &Vec<Expr>) -> DisplayWrap {
        self.write("[");
        for (i, el) in elements.iter().enumerate() {
            if i > 0 {
                self.write(", ");
            }
            self.visit_expr(el);
        }
        self.write("]");
        DisplayWrap::Unwrapped
    }

    fn visit_path(&mut self, path: &Vec<&str>) -> DisplayWrap {
        for (i, part) in path.iter().enumerate() {
            if i > 0 {
                self.write("::");
            }
            self.write(part);
        }
        DisplayWrap::Unwrapped
    }

    fn visit_var(&mut self, s: &str) -> DisplayWrap {
        if self.locals.contains(s) {
            self.write(s);
        } else {
            self.write(&format!("self.{}", s));
        }
        DisplayWrap::Unwrapped
    }

    fn visit_str_lit(&mut self, s: &str) -> DisplayWrap {
        self.write(&format!("\"{}\"", s));
        DisplayWrap::Unwrapped
    }

    fn visit_num_lit(&mut self, s: &str) -> DisplayWrap {
        self.write(s);
        DisplayWrap::Unwrapped
    }

    fn visit_target_single<'t>(&mut self, name: &'t str) -> Vec<&'t str> {
        vec![name]
    }

    fn visit_target<'t>(&mut self, target: &'t Target) -> Vec<&'t str> {
        match *target {
            Target::Name(s) => { self.visit_target_single(s) },
        }
    }

    /* Helper methods for dealing with whitespace nodes */

    fn handle_ws(&mut self, ws: &WS) {
        self.flush_ws(ws);
        self.prepare_ws(ws);
    }

    fn flush_ws(&mut self, ws: &WS) {
        if self.next_ws.is_some() && !ws.0 {
            let val = self.next_ws.unwrap();
            if !val.is_empty() {
                self.writeln(&format!("writer.write_str({:#?})?;",
                                      val));
            }
        }
        self.next_ws = None;
    }

    fn prepare_ws(&mut self, ws: &WS) {
        self.skip_ws = ws.1;
    }

    /* Helper methods for writing to internal buffer */

    fn writeln(&mut self, s: &str) {
        if s.is_empty() {
            return;
        }
        if s == "}" {
            self.dedent();
        }
        self.write(s);
        if s.ends_with('{') {
            self.indent();
        }
        self.buf.push('\n');
        self.start = true;
    }

    fn write(&mut self, s: &str) {
        if self.start {
            for _ in 0..(self.indent * 4) {
                self.buf.push(' ');
            }
            self.start = false;
        }
        self.buf.push_str(s);
    }

    fn indent(&mut self) {
        self.indent += 1;
    }

    fn dedent(&mut self) {
        if self.indent == 0 {
            panic!("dedent() called while indentation == 0");
        }
        self.indent -= 1;
    }
}

struct SetChain<'a, T: 'a> where T: cmp::Eq + hash::Hash {
    parent: Option<&'a SetChain<'a, T>>,
    scopes: Vec<HashSet<T>>,
}

impl<'a, T: 'a> SetChain<'a, T> where T: cmp::Eq + hash::Hash {
    fn new() -> SetChain<'a, T> {
        SetChain { parent: None, scopes: vec![HashSet::new()] }
    }
    fn with_parent<'p>(parent: &'p SetChain<T>) -> SetChain<'p, T> {
        SetChain { parent: Some(parent), scopes: vec![HashSet::new()] }
    }
    fn contains(&self, val: T) -> bool {
        self.scopes.iter().rev().any(|set| set.contains(&val)) ||
            match self.parent {
                Some(set) => set.contains(val),
                None => false,
            }
    }
    fn insert(&mut self, val: T) {
        self.scopes.last_mut().unwrap().insert(val);
    }
    fn push(&mut self) {
        self.scopes.push(HashSet::new());
    }
    fn pop(&mut self) {
        self.scopes.pop().unwrap();
        assert!(!self.scopes.is_empty());
    }
}

enum AstLevel {
    Top,
    Nested,
}

enum DisplayWrap {
    Wrapped,
    Unwrapped,
}

type MacroMap<'a> = HashMap<(Option<&'a str>, &'a str), &'a Macro<'a>>;
