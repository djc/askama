use parser::{Cond, Expr, Node, Target, WS};
use std::str;
use std::collections::HashSet;
use syn;
use quote::{Tokens, ToTokens};

fn path_as_identifier(s: &str) -> String {
    let mut res = String::new();
    for c in s.chars() {
        if c.is_alphanumeric() {
            res.push(c);
        } else {
            res.push_str(&format!("{:x}", c as u32));
        }
    }
    res
}

struct Generator<'a> {
    buf: String,
    indent: u8,
    start: bool,
    locals: HashSet<String>,
    next_ws: Option<&'a str>,
    skip_ws: bool,
}

impl<'a> Generator<'a> {

    fn new() -> Generator<'a> {
        Generator {
            buf: String::new(),
            indent: 0,
            start: true,
            locals: HashSet::new(),
            next_ws: None,
            skip_ws: false,
        }
    }

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

    fn flush_ws(&mut self, ws: &WS) {
        if self.next_ws.is_some() && !ws.0 {
            let val = self.next_ws.unwrap();
            if !val.is_empty() {
                self.writeln(&format!("writer.write_str({:#?}).unwrap();",
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
        } else {
            self.write(&format!("::askama::filters::{}(&", name));
        }

        for (i, arg) in args.iter().enumerate() {
            if i > 0 {
                self.write(", &");
            }
            self.visit_expr(arg);
        }
        self.write(")");
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

    fn visit_call(&mut self, obj: &Expr, method: &str, args: &[Expr]) {
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
            Expr::Call(ref obj, method, ref args) =>
                self.visit_call(obj, method, args),
        }
    }

    fn visit_target_single(&mut self, name: &str) -> Vec<String> {
        vec![name.to_string()]
    }

    fn visit_target(&mut self, target: &Target) -> Vec<String> {
        match *target {
            Target::Name(s) => { self.visit_target_single(s) },
        }
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
                self.writeln(&format!("writer.write_str({:#?}).unwrap();",
                                      lws));
            }
        }
        if !val.is_empty() {
            self.writeln(&format!("writer.write_str({:#?}).unwrap();", val));
        }
        if !rws.is_empty() {
            self.next_ws = Some(rws);
        }
    }

    fn write_expr(&mut self, ws: &WS, s: &Expr) {
        self.handle_ws(ws);
        self.write("writer.write_fmt(format_args!(\"{}\", ");
        self.visit_expr(s);
        self.writeln(")).unwrap();");
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
            self.handle(nodes);
        }
        self.handle_ws(ws);
        self.writeln("}");
    }

    fn write_loop(&mut self, ws1: &WS, var: &Target, iter: &Expr,
                  body: &'a [Node], ws2: &WS) {
        self.handle_ws(ws1);
        self.write("for (_loop_index, ");
        let targets = self.visit_target(var);
        for name in &targets {
            self.locals.insert(name.clone());
            self.write(name);
        }
        self.write(") in (&");
        self.visit_expr(iter);
        self.writeln(").into_iter().enumerate() {");

        self.handle(body);
        self.handle_ws(ws2);
        self.writeln("}");
        for name in &targets {
            self.locals.remove(name);
        }
    }

    fn write_block(&mut self, ws1: &WS, name: &str, ws2: &WS) {
        self.flush_ws(ws1);
        self.writeln(&format!("timpl.render_block_{}_to(writer);", name));
        self.prepare_ws(ws2);
    }

    fn write_block_def(&mut self, ws1: &WS, name: &str, nodes: &'a [Node],
                       ws2: &WS) {
        self.writeln("#[allow(unused_variables)]");
        self.writeln(&format!(
            "fn render_block_{}_to(&self, writer: &mut ::std::fmt::Write) {{",
            name));
        self.prepare_ws(ws1);
        self.handle(nodes);
        self.flush_ws(ws2);
        self.writeln("}");
    }

    fn handle(&mut self, nodes: &'a [Node]) {
        for n in nodes {
            match *n {
                Node::Lit(lws, val, rws) => { self.write_lit(lws, val, rws); }
                Node::Comment() => {},
                Node::Expr(ref ws, ref val) => { self.write_expr(ws, val); },
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
                Node::Extends(_) => {
                    panic!("no extends or block definition allowed in content");
                },
            }
        }
    }

    fn write_header(&mut self, ast: &syn::DeriveInput,
                    trait_suffix: Option<&str>) {
        let mut full_anno = Tokens::new();
        let mut orig_anno = Tokens::new();
        let need_anno = ast.generics.lifetimes.len() > 0 ||
                        ast.generics.ty_params.len() > 0;
        if need_anno {
            full_anno.append("<");
            orig_anno.append("<");
        }

        let mut sep = false;
        for lt in &ast.generics.lifetimes {
            if sep {
                full_anno.append(",");
                orig_anno.append(",");
            }
            lt.to_tokens(&mut full_anno);
            lt.to_tokens(&mut orig_anno);
            sep = true;
        }

        for param in &ast.generics.ty_params {
            if sep {
                full_anno.append(",");
                orig_anno.append(",");
            }
            let mut impl_param = param.clone();
            impl_param.default = None;
            impl_param.to_tokens(&mut full_anno);
            param.ident.to_tokens(&mut orig_anno);
            sep = true;
        }

        if need_anno {
            full_anno.append(">");
            orig_anno.append(">");
        }

        let mut where_clause = Tokens::new();
        ast.generics.where_clause.to_tokens(&mut where_clause);
        let name = if let Some(suffix) = trait_suffix {
            format!("TraitFrom{}", suffix)
        } else {
            "::askama::Template".to_string()
        };
        self.writeln(&format!("impl{} {} for {}{}{} {{",
                              full_anno.as_str(), &name, ast.ident.as_ref(),
                              orig_anno.as_str(), where_clause.as_str()));
    }

    fn impl_template(&mut self, ast: &syn::DeriveInput, nodes: &'a [Node]) {
        self.write_header(ast, None);
        self.writeln("fn render_to(&self, writer: &mut ::std::fmt::Write) {");
        self.handle(nodes);
        self.flush_ws(&WS(false, false));
        self.writeln("}");
        self.writeln("}");
    }

    fn impl_trait(&mut self, ast: &syn::DeriveInput, base: &str,
                  blocks: &'a [Node], nodes: Option<&'a [Node]>) {
        self.write_header(ast, Some(&path_as_identifier(base)));
        self.handle(blocks);

        self.writeln("#[allow(unused_variables)]");
        let trait_name = format!("TraitFrom{}", path_as_identifier(base));
        self.writeln(&format!(
            "fn render_trait_to(&self, timpl: &{}, writer: &mut ::std::fmt::Write) {{",
            trait_name));

        if let Some(nodes) = nodes {
            self.handle(nodes);
            self.flush_ws(&WS(false, false));
        } else {
            self.writeln("self._parent.render_trait_to(self, writer);");
        }

        self.writeln("}");
        self.flush_ws(&WS(false, false));
        self.writeln("}");
    }

    fn impl_template_for_trait(&mut self, ast: &syn::DeriveInput, derived: bool) {
        self.write_header(ast, None);
        self.writeln("fn render_to(&self, writer: &mut ::std::fmt::Write) {");
        if derived {
            self.writeln("self._parent.render_trait_to(self, writer);");
        } else {
            self.writeln("self.render_trait_to(self, writer);");
        }
        self.writeln("}");
        self.writeln("}");
    }

    fn define_trait(&mut self, path: &str, block_names: &[&str]) {
        let trait_name = format!("TraitFrom{}", path_as_identifier(path));
        self.writeln(&format!("trait {} {{", &trait_name));

        for bname in block_names {
            self.writeln(&format!(
                "fn render_block_{}_to(&self, writer: &mut ::std::fmt::Write);",
                bname));
        }
        self.writeln(&format!(
            "fn render_trait_to(&self, timpl: &{}, writer: &mut ::std::fmt::Write);",
            trait_name));

        self.writeln("}");
    }

    fn result(self) -> String {
        self.buf
    }

}

pub fn generate(ast: &syn::DeriveInput, path: &str, mut nodes: Vec<Node>) -> String {
    let mut base: Option<Expr> = None;
    let mut blocks = Vec::new();
    let mut block_names = Vec::new();
    let mut content = Vec::new();
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
            _ => { content.push(n); },
        }
    }

    let mut gen = Generator::new();
    if !blocks.is_empty() {
        if base.is_none() {
            gen.define_trait(path, &block_names);
        }
        let tmpl_path = match base {
            Some(Expr::StrLit(base_path)) => { base_path },
            _ => { path },
        };
        let trait_nodes = if base.is_none() { Some(&content[..]) } else { None };
        gen.impl_trait(ast, tmpl_path, &blocks, trait_nodes);
        gen.impl_template_for_trait(ast, base.is_some());
    } else {
        gen.impl_template(ast, &content);
    }
    gen.result()
}
