use parser::{self, Cond, Expr, Node, Target, WS};
use path;

use quote::{Tokens, ToTokens};

use std::{cmp, hash, str};
use std::path::Path;
use std::collections::{HashMap, HashSet};

use syn;

pub fn generate(ast: &syn::DeriveInput, path: &Path, mut nodes: Vec<Node>) -> String {
    let mut base: Option<Expr> = None;
    let mut blocks = Vec::new();
    let mut block_names = Vec::new();
    let mut content = Vec::new();
    let mut macros = HashMap::new();
    for n in nodes.drain(..) {
        match n {
            Node::Extends(path) => {
                match base {
                    Some(_) => panic!("multiple extend blocks found"),
                    None => { base = Some(path); },
                }
            },
            Node::BlockDef(ws1, name, _, ws2) => {
                blocks.push(n);
                block_names.push(name);
                content.push(Node::Block(ws1, name, ws2));
            },
            Node::Macro(ws1, name, params, contents, ws2) => {
                macros.insert(name, (ws1, name, params, contents, ws2));
            },
            _ => { content.push(n); },
        }
    }

    let mut gen = Generator::default(&macros);
    if !blocks.is_empty() {
        let trait_name = trait_name_for_path(&base, path);
        if base.is_none() {
            gen.define_trait(&trait_name, &block_names);
        } else {
            let parent_type = get_parent_type(ast)
                .expect("expected field '_parent' in extending template struct");
            gen.deref_to_parent(ast, &parent_type);
        }

        let trait_nodes = if base.is_none() { Some(&content[..]) } else { None };
        gen.impl_trait(ast, &trait_name, &blocks, trait_nodes);
        gen.impl_template_for_trait(ast, base.is_some());
    } else {
        gen.impl_template(ast, &content);
    }
    gen.impl_display(ast);
    if cfg!(feature = "iron") {
        gen.impl_modifier_response(ast);
    }
    if cfg!(feature = "rocket") {
        gen.impl_responder(ast, path);
    }
    gen.result()
}

fn trait_name_for_path(base: &Option<Expr>, path: &Path) -> String {
    let rooted_path = match *base {
        Some(Expr::StrLit(user_path)) => {
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
    macros: &'a MacroMap<'a>,
}

impl<'a> Generator<'a> {

    fn new<'n>(macros: &'n MacroMap, locals: SetChain<'n, &'n str>, indent: u8) -> Generator<'n> {
        Generator {
            buf: String::new(),
            indent: indent,
            start: true,
            locals: locals,
            next_ws: None,
            skip_ws: false,
            macros: macros,
        }
    }

    fn default<'n>(macros: &'n MacroMap) -> Generator<'n> {
        Self::new(macros, SetChain::new(), 0)
    }

    fn child<'n>(&'n mut self) -> Generator<'n> {
        let locals = SetChain::with_parent(&self.locals);
        Self::new(self.macros, locals, self.indent)
    }

    /* Helper methods for writing to internal buffer */

    fn indent(&mut self) {
        self.indent += 1;
    }

    fn dedent(&mut self) {
        self.indent -= 1;
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

    /* Helper methods for dealing with whitespace nodes */

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

    fn handle_ws(&mut self, ws: &WS) {
        self.flush_ws(ws);
        self.prepare_ws(ws);
    }

    /* Visitor methods for expression types */

    fn visit_num_lit(&mut self, s: &str) {
        self.write(s);
    }

    fn visit_str_lit(&mut self, s: &str) {
        self.write(&format!("\"{}\"", s));
    }

    fn visit_var(&mut self, s: &str) {
        if self.locals.contains(s) {
            self.write(s);
        } else {
            self.write(&format!("self.{}", s));
        }
    }

    fn visit_attr(&mut self, obj: &Expr, attr: &str) {
        if let Expr::Var(name) = *obj {
            if name == "loop" {
                self.write("_loop_index");
                if attr == "index" {
                    self.write(" + 1");
                    return;
                } else if attr == "index0" {
                    return;
                } else {
                    panic!("unknown loop variable");
                }
            }
        }
        self.visit_expr(obj);
        self.write(&format!(".{}", attr));
    }

    fn visit_filter(&mut self, name: &str, args: &[Expr]) {
        if name == "format" {
            self.write("format!(");
        } else if BUILT_IN_FILTERS.contains(&name) {
            self.write(&format!("::askama::filters::{}(&", name));
        } else {
            self.write(&format!("filters::{}(&", name));
        }

        for (i, arg) in args.iter().enumerate() {
            if i > 0 {
                self.write(", &");
            }
            self.visit_expr(arg);
        }
        self.write(")");
        if name != "format" {
            self.write("?");
        }
    }

    fn visit_binop(&mut self, op: &str, left: &Expr, right: &Expr) {
        self.visit_expr(left);
        self.write(&format!(" {} ", op));
        self.visit_expr(right);
    }

    fn visit_group(&mut self, inner: &Expr) {
        self.write("(");
        self.visit_expr(inner);
        self.write(")");
    }

    fn visit_method_call(&mut self, obj: &Expr, method: &str, args: &[Expr]) {
        self.visit_expr(obj);
        self.write(&format!(".{}(", method));
        for (i, arg) in args.iter().enumerate() {
            if i > 0 {
                self.write(", ");
            }
            self.visit_expr(arg);
        }
        self.write(")");
    }

    fn visit_expr(&mut self, expr: &Expr) {
        match *expr {
            Expr::NumLit(s) => self.visit_num_lit(s),
            Expr::StrLit(s) => self.visit_str_lit(s),
            Expr::Var(s) => self.visit_var(s),
            Expr::Attr(ref obj, name) => self.visit_attr(obj, name),
            Expr::Filter(name, ref args) => self.visit_filter(name, args),
            Expr::BinOp(op, ref left, ref right) =>
                self.visit_binop(op, left, right),
            Expr::Group(ref inner) => self.visit_group(inner),
            Expr::MethodCall(ref obj, method, ref args) =>
                self.visit_method_call(obj, method, args),
        }
    }

    fn visit_target_single<'t>(&mut self, name: &'t str) -> Vec<&'t str> {
        vec![name]
    }

    fn visit_target<'t>(&mut self, target: &'t Target) -> Vec<&'t str> {
        match *target {
            Target::Name(s) => { self.visit_target_single(s) },
        }
    }

    /* Helper methods for handling node types */

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

    fn write_expr(&mut self, ws: &WS, s: &Expr) {
        self.handle_ws(ws);

        let is_safe = match *s {
            Expr::Filter(name, _) if name == "safe" => true,
            _ => false
        };

        self.write("writer.write_fmt(");
        self.write("format_args!(\"{}\", ");

        if !is_safe {
            self.write("::askama::filters::escape(&(");
        }

        self.visit_expr(s);

        if !is_safe {
            self.write("))?");
        }

        self.write(")");
        self.writeln(")?;");
    }

    fn write_call(&mut self, ws: &WS, name: &str, args: &[Expr]) {
        self.handle_ws(ws);
        let def = self.macros.get(name).expect(&format!("macro '{}' not found", name));
        self.locals.push();
        self.writeln("{");
        self.prepare_ws(&def.0);
        for (i, arg) in def.2.iter().enumerate() {
            self.write(&format!("let {} = &", arg));
            self.locals.insert(arg);
            self.visit_expr(&args.get(i)
                .expect(&format!("macro '{}' takes more than {} arguments", name, i)));
            self.writeln(";");
        }
        self.handle(&def.3);
        self.flush_ws(&def.4);
        self.writeln("}");
        self.locals.pop();
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

    fn write_cond(&mut self, conds: &'a [Cond], ws: &WS) {
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
            self.handle(nodes);
            self.locals.pop();
        }
        self.handle_ws(ws);
        self.writeln("}");
    }

    fn write_loop(&mut self, ws1: &WS, var: &'a Target, iter: &Expr,
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

        self.handle(body);
        self.handle_ws(ws2);
        self.writeln("}");
        self.locals.pop();
    }

    fn write_block(&mut self, ws1: &WS, name: &str, ws2: &WS) {
        self.flush_ws(ws1);
        self.writeln(&format!("timpl.render_block_{}_into(writer)?;", name));
        self.prepare_ws(ws2);
    }

    fn write_block_def(&mut self, ws1: &WS, name: &str, nodes: &'a [Node],
                       ws2: &WS) {
        self.writeln("#[allow(unused_variables)]");
        self.writeln(&format!(
            "fn render_block_{}_into(&self, writer: &mut ::std::fmt::Write) \
             -> ::askama::Result<()> {{",
            name));
        self.prepare_ws(ws1);

        self.locals.push();
        self.handle(nodes);
        self.locals.pop();

        self.flush_ws(ws2);
        self.writeln("Ok(())");
        self.writeln("}");
    }

    fn handle_include(&mut self, ws: &WS, path: &str) {
        self.prepare_ws(ws);
        let path = path::find_template_from_path(&path, None);
        let src = path::get_template_source(&path);
        let nodes = parser::parse(&src);
        let nested = {
            let mut gen = self.child();
            gen.handle(&nodes);
            gen.result()
        };
        self.buf.push_str(&nested);
        self.flush_ws(ws);
    }

    fn handle(&mut self, nodes: &'a [Node]) {
        for n in nodes {
            match *n {
                Node::Lit(lws, val, rws) => { self.write_lit(lws, val, rws); }
                Node::Comment() => {},
                Node::Expr(ref ws, ref val) => { self.write_expr(ws, val); },
                Node::LetDecl(ref ws, ref var) => { self.write_let_decl(ws, var); },
                Node::Let(ref ws, ref var, ref val) => { self.write_let(ws, var, val); },
                Node::Cond(ref conds, ref ws) => {
                    self.write_cond(conds, ws);
                },
                Node::Loop(ref ws1, ref var, ref iter, ref body, ref ws2) => {
                    self.write_loop(ws1, var, iter, body, ws2);
                },
                Node::Block(ref ws1, name, ref ws2) => {
                    self.write_block(ws1, name, ws2);
                },
                Node::BlockDef(ref ws1, name, ref block_nodes, ref ws2) => {
                    self.write_block_def(ws1, name, block_nodes, ws2);
                }
                Node::Include(ref ws, ref path) => {
                    self.handle_include(ws, path);
                },
                Node::Call(ref ws, name, ref args) => self.write_call(ws, name, args),
                Node::Macro(_, _, _, _, _) |
                Node::Extends(_) => {
                    panic!("no extends or macros allowed in content");
                },
            }
        }
    }

    // Writes header for the `impl` for `TraitFromPathName` or `Template`
    // for the given context struct.
    fn write_header(&mut self, ast: &syn::DeriveInput, target: &str, extra_anno: &[&str]) {
        let mut full_anno = Tokens::new();
        let mut orig_anno = Tokens::new();
        let need_anno = ast.generics.lifetimes.len() > 0 ||
                        ast.generics.ty_params.len() > 0 ||
                        extra_anno.len() > 0;
        if need_anno {
            full_anno.append("<");
            orig_anno.append("<");
        }

        let (mut full_sep, mut orig_sep) = (false, false);
        for lt in &ast.generics.lifetimes {
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

        for param in &ast.generics.ty_params {
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
        ast.generics.where_clause.to_tokens(&mut where_clause);
        self.writeln(&format!("impl{} {} for {}{}{} {{",
                              full_anno.as_str(), target, ast.ident.as_ref(),
                              orig_anno.as_str(), where_clause.as_str()));
    }

    // Implement `Template` for the given context struct.
    fn impl_template(&mut self, ast: &syn::DeriveInput, nodes: &'a [Node]) {
        self.write_header(ast, "::askama::Template", &vec![]);
        self.writeln("fn render_into(&self, writer: &mut ::std::fmt::Write) -> \
                      ::askama::Result<()> {");
        self.handle(nodes);
        self.flush_ws(&WS(false, false));
        self.writeln("Ok(())");
        self.writeln("}");
        self.writeln("}");
    }

    // Implement `Display` for the given context struct.
    fn impl_display(&mut self, ast: &syn::DeriveInput) {
        self.write_header(ast, "::std::fmt::Display", &vec![]);
        self.writeln("fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {");
        self.writeln("self.render_into(f).map_err(|_| ::std::fmt::Error {})");
        self.writeln("}");
        self.writeln("}");
    }

    // Implement `Deref<Parent>` for an inheriting context struct.
    fn deref_to_parent(&mut self, ast: &syn::DeriveInput, parent_type: &syn::Ty) {
        self.write_header(ast, "::std::ops::Deref", &vec![]);
        let mut tokens = Tokens::new();
        parent_type.to_tokens(&mut tokens);
        self.writeln(&format!("type Target = {};", tokens.as_str()));
        self.writeln("fn deref(&self) -> &Self::Target {");
        self.writeln("&self._parent");
        self.writeln("}");
        self.writeln("}");
    }

    // Implement `TraitFromPathName` for the given context struct.
    fn impl_trait(&mut self, ast: &syn::DeriveInput, trait_name: &str,
                  blocks: &'a [Node], nodes: Option<&'a [Node]>) {
        self.write_header(ast, &trait_name, &vec![]);
        self.handle(blocks);

        self.writeln("#[allow(unused_variables)]");
        self.writeln(&format!(
            "fn render_trait_into(&self, timpl: &{}, writer: &mut ::std::fmt::Write) \
             -> ::askama::Result<()> {{",
            trait_name));

        if let Some(nodes) = nodes {
            self.handle(nodes);
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
    fn impl_template_for_trait(&mut self, ast: &syn::DeriveInput, derived: bool) {
        self.write_header(ast, "::askama::Template", &vec![]);
        self.writeln("fn render_into(&self, writer: &mut ::std::fmt::Write) \
                      -> ::askama::Result<()> {");
        if derived {
            self.writeln("self._parent.render_trait_into(self, writer)?;");
        } else {
            self.writeln("self.render_trait_into(self, writer)?;");
        }
        self.writeln("Ok(())");
        self.writeln("}");
        self.writeln("}");
    }

    // Defines the `TraitFromPathName` trait.
    fn define_trait(&mut self, trait_name: &str, block_names: &[&str]) {
        self.writeln(&format!("trait {} {{", &trait_name));

        for bname in block_names {
            self.writeln(&format!(
                "fn render_block_{}_into(&self, writer: &mut ::std::fmt::Write) \
                -> ::askama::Result<()>;",
                bname));
        }
        self.writeln(&format!(
            "fn render_trait_into(&self, timpl: &{}, writer: &mut ::std::fmt::Write) \
             -> ::askama::Result<()>;",
            trait_name));
        self.writeln("}");
    }

    // Implement iron's Modifier<Response> if enabled
    fn impl_modifier_response(&mut self, ast: &syn::DeriveInput) {
        self.write_header(ast, "::askama::iron::Modifier<::askama::iron::Response>", &vec![]);
        self.writeln("fn modify(self, res: &mut ::askama::iron::Response) {");
        self.writeln("res.body = Some(Box::new(self.render().unwrap().into_bytes()));");
        self.writeln("}");
        self.writeln("}");
    }

    // Implement Rocket's `Responder`.
    fn impl_responder(&mut self, ast: &syn::DeriveInput, path: &Path) {
        self.write_header(ast, "::askama::rocket::Responder<'r>", &vec!["'r"]);
        self.writeln("fn respond_to(self, _: &::askama::rocket::Request) \
                      -> ::std::result::Result<\
                         ::askama::rocket::Response<'r>, ::askama::rocket::Status> {");
        let ext = match path.extension() {
            Some(s) => s.to_str().unwrap(),
            None => "txt",
        };
        self.writeln("::askama::rocket::Response::build()");
        self.indent();
        self.writeln(&format!(".header(::askama::rocket::ContentType::from_extension({:?})\
                               .unwrap())", ext));
        self.writeln(".sized_body(::std::io::Cursor::new(self.render().unwrap()))");
        self.writeln(".ok()");
        self.dedent();
        self.writeln("}");
        self.writeln("}");
    }

    fn result(self) -> String {
        self.buf
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
        assert!(self.scopes.len() > 0);
    }
}

type MacroMap<'a> = HashMap<&'a str, (WS, &'a str, Vec<&'a str>, Vec<Node<'a>>, WS)>;

const BUILT_IN_FILTERS: [&str; 10] = [
    "e",
    "escape",
    "safe",
    "format",
    "lower",
    "lowercase",
    "trim",
    "upper",
    "uppercase",
    "json", // Optional feature; reserve the name anyway
];
