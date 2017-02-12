use parser::{Cond, Expr, Node, Target, WS};
use std::str;
use std::collections::HashSet;
use syn;

fn annotations(generics: &syn::Generics) -> String {
    if generics.lifetimes.len() < 1 {
        return String::new();
    }
    let mut res = String::new();
    res.push('<');
    for lt in &generics.lifetimes {
        res.push_str(lt.lifetime.ident.as_ref());
    }
    res.push('>');
    res
}

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
        self.write(s);
        self.buf.push('\n');
        self.start = true;
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

    fn visit_filter(&mut self, name: &str, val: &Expr) {
        self.write(&format!("askama::filters::{}(&", name));
        self.visit_expr(val);
        self.write(")");
    }

    fn visit_compare(&mut self, op: &str, left: &Expr, right: &Expr) {
        self.visit_expr(left);
        self.write(&format!(" {} ", op));
        self.visit_expr(right);
    }

    fn visit_expr(&mut self, expr: &Expr) {
        match *expr {
            Expr::StrLit(s) => self.visit_str_lit(s),
            Expr::Var(s) => self.visit_var(s),
            Expr::Filter(name, ref val) => self.visit_filter(name, val),
            Expr::Compare(op, ref left, ref right) =>
                self.visit_compare(op, left, right),
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

    fn flush_ws(&mut self, ws: &WS) {
        if self.next_ws.is_some() && !ws.0 {
            let val = self.next_ws.unwrap();
            if !val.is_empty() {
                self.writeln(&format!("writer.write_str({:#?}).unwrap();",
                                      val));
            }
        } else if self.next_ws.is_some() {
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
                        self.write("} else if ");
                    }
                    self.visit_expr(expr);
                },
                None => { self.writeln("} else"); },
            }
            self.writeln(" {");
            self.indent();
            self.handle(nodes);
            self.dedent();
        }
        self.handle_ws(ws);
        self.writeln("}");
    }

    fn write_loop(&mut self, ws1: &WS, var: &Target, iter: &Expr,
                  body: &'a [Node], ws2: &WS) {

        self.handle_ws(ws1);
        self.write("for ");
        let targets = self.visit_target(var);
        for name in &targets {
            self.locals.insert(name.clone());
            self.write(name);
        }
        self.write(" in &");
        self.visit_expr(iter);
        self.writeln(" {");

        self.indent();
        self.handle(body);
        self.handle_ws(ws2);
        self.dedent();
        self.writeln("}");
        for name in &targets {
            self.locals.remove(name);
        }
    }

    fn write_block(&mut self, ws1: &WS, name: &str, ws2: &WS) {
        self.flush_ws(ws1);
        self.writeln(&format!("self.render_block_{}_to(writer);", name));
        self.prepare_ws(ws2);
    }

    fn write_block_def(&mut self, ws1: &WS, name: &str, nodes: &'a [Node],
                       ws2: &WS) {
        self.writeln("#[allow(unused_variables)]");
        self.writeln(&format!(
            "fn impl_block_{}_to(&self, writer: &mut std::fmt::Write) {{",
            name));
        self.indent();
        self.prepare_ws(ws1);
        self.handle(nodes);
        self.flush_ws(ws2);
        self.dedent();
        self.writeln("}");
    }

    fn handle(&mut self, nodes: &'a [Node]) {
        for n in nodes {
            match *n {
                Node::Lit(lws, val, rws) => { self.write_lit(lws, val, rws); }
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

    fn path_based_name(&mut self, ast: &syn::DeriveInput, path: &str) {
        let encoded = path_as_identifier(path);
        let original = ast.ident.as_ref();
        let anno = annotations(&ast.generics);
        self.writeln("#[allow(dead_code, non_camel_case_types)]");
        let s = format!("type TemplateFrom{}{} = {}{};",
                        encoded, &anno, original, &anno);
        self.writeln(&s);
    }

    fn struct_impl(&mut self, ast: &syn::DeriveInput, blocks: &'a [Node]) {
        let anno = annotations(&ast.generics);
        self.writeln(&format!("impl{} {}{} {{", anno, ast.ident.as_ref(),
                              anno));
        self.indent();
        self.handle(blocks);
        self.dedent();
        self.writeln("}");
    }

    fn template_impl(&mut self, ast: &syn::DeriveInput, nodes: &'a [Node]) {
        let anno = annotations(&ast.generics);
        self.writeln(&format!("impl{} askama::Template for {}{} {{",
                              anno, ast.ident.as_ref(), anno));
        self.indent();

        self.writeln("fn render_to(&self, writer: &mut std::fmt::Write) {");
        self.indent();
        self.handle(nodes);
        self.flush_ws(&WS(false, false));
        self.dedent();
        self.writeln("}");

        self.dedent();
        self.writeln("}");
    }

    fn trait_impl(&mut self, ast: &syn::DeriveInput, base: &str,
                  block_names: &[&str]) {
        let anno = annotations(&ast.generics);
        self.writeln(&format!("impl{} TraitFrom{} for {}{} {{",
                              anno, path_as_identifier(base),
                              ast.ident.as_ref(), anno));
        self.indent();

        for bname in block_names {
            self.writeln(&format!(
                "fn render_block_{}_to(&self, writer: &mut std::fmt::Write) {{",
                bname));
            self.indent();
            self.writeln(&format!("self.impl_block_{}_to(writer);", bname));
            self.dedent();
            self.writeln("}");
        }

        self.flush_ws(&WS(false, false));
        self.dedent();
        self.writeln("}");
    }

    fn trait_based_impl(&mut self, ast: &syn::DeriveInput) {
        let anno = annotations(&ast.generics);
        self.writeln(&format!("impl{} askama::Template for {}{} {{",
                              anno, ast.ident.as_ref(), anno));
        self.indent();

        self.writeln("fn render_to(&self, writer: &mut std::fmt::Write) {");
        self.indent();
        self.writeln("self.render_trait_to(writer);");
        self.dedent();
        self.writeln("}");

        self.dedent();
        self.writeln("}");
    }

    fn template_trait(&mut self, ast: &syn::DeriveInput, path: &str,
                      block_names: &[&str], nodes: &'a [Node]) {
        let anno = annotations(&ast.generics);
        self.writeln(&format!("trait{} TraitFrom{}{} {{", anno,
                              path_as_identifier(path), anno));
        self.indent();

        for bname in block_names {
            self.writeln(&format!(
                "fn render_block_{}_to(&self, writer: &mut std::fmt::Write);",
                bname));
        }

        self.writeln("fn render_trait_to(&self, writer: &mut std::fmt::Write) {");
        self.indent();
        self.handle(nodes);
        self.flush_ws(&WS(false, false));
        self.dedent();
        self.writeln("}");

        self.dedent();
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
    gen.path_based_name(ast, path);
    if !blocks.is_empty() {
        gen.struct_impl(ast, &blocks);
        if base.is_none() {
            gen.template_trait(ast, path, &block_names, &content);
        }
        let tmpl_path = match base {
            Some(Expr::StrLit(base_path)) => { base_path },
            _ => { path },
        };
        gen.trait_impl(ast, tmpl_path, &block_names);
        gen.trait_based_impl(ast);
    } else {
        gen.template_impl(ast, &content);
    }
    gen.result()
}
