use parser::{Cond, Expr, Node, Target};
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

struct Generator {
    buf: String,
    indent: u8,
    start: bool,
    locals: HashSet<String>,
}

impl Generator {

    fn new() -> Generator {
        Generator {
            buf: String::new(),
            indent: 0,
            start: true,
            locals: HashSet::new(),
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

    fn write_lit(&mut self, s: &str) {
        self.write("writer.write_str(");
        self.write(&format!("{:#?}", s));
        self.writeln(").unwrap();");
    }

    fn write_expr(&mut self, s: &Expr) {
        self.write("writer.write_fmt(format_args!(\"{}\", ");
        self.visit_expr(s);
        self.writeln(")).unwrap();");
    }

    fn write_cond(&mut self, conds: &[Cond]) {
        for (i, &(_, ref cond, ref nodes)) in conds.iter().enumerate() {
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
        self.writeln("}");
    }

    fn write_loop(&mut self, var: &Target, iter: &Expr, body: &[Node]) {

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
        self.dedent();
        self.writeln("}");
        for name in &targets {
            self.locals.remove(name);
        }
    }

    fn write_block(&mut self, name: &str) {
        self.writeln(&format!("self.render_block_{}_into(writer);", name));
    }

    fn write_block_def(&mut self, name: &str, nodes: &[Node]) {
        self.writeln("#[allow(unused_variables)]");
        self.writeln(&format!(
            "fn render_block_{}_into(&self, writer: &mut std::fmt::Write) {{",
            name));
        self.indent();
        self.handle(nodes);
        self.dedent();
        self.writeln("}");
    }

    fn handle(&mut self, nodes: &[Node]) {
        for n in nodes {
            match *n {
                Node::Lit(lws, val, rws) => {
                    self.write_lit(lws);
                    self.write_lit(val);
                    self.write_lit(rws);
                },
                Node::Expr(_, ref val) => { self.write_expr(val); },
                Node::Cond(ref conds, _) => { self.write_cond(conds); },
                Node::Loop(_, ref var, ref iter, ref body, _) => {
                    self.write_loop(var, iter, body);
                },
                Node::Block(_, name, _) => { self.write_block(name) },
                Node::BlockDef(_, name, ref block_nodes, _) => {
                    self.write_block_def(name, block_nodes);
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

    fn template_impl(&mut self, ast: &syn::DeriveInput, nodes: &[Node]) {
        let anno = annotations(&ast.generics);
        self.writeln(&format!("impl{} askama::Template for {}{} {{",
                              anno, ast.ident.as_ref(), anno));
        self.indent();

        self.writeln("fn render_into(&self, writer: &mut std::fmt::Write) {");
        self.indent();
        self.handle(nodes);
        self.dedent();
        self.writeln("}");

        self.dedent();
        self.writeln("}");
    }

    fn trait_impl(&mut self, ast: &syn::DeriveInput, base: &str, blocks: &[Node]) {
        let anno = annotations(&ast.generics);
        self.writeln(&format!("impl{} TraitFrom{} for {}{} {{",
                              anno, path_as_identifier(base),
                              ast.ident.as_ref(), anno));
        self.indent();
        self.handle(blocks);
        self.dedent();
        self.writeln("}");
    }

    fn trait_based_impl(&mut self, ast: &syn::DeriveInput) {
        let anno = annotations(&ast.generics);
        self.writeln(&format!("impl{} askama::Template for {}{} {{",
                              anno, ast.ident.as_ref(), anno));
        self.indent();

        self.writeln("fn render_into(&self, writer: &mut std::fmt::Write) {");
        self.indent();
        self.writeln("self.render_trait_into(writer);");
        self.dedent();
        self.writeln("}");

        self.dedent();
        self.writeln("}");
    }

    fn template_trait(&mut self, ast: &syn::DeriveInput, path: &str,
                      blocks: &[Node], nodes: &[Node]) {
        let anno = annotations(&ast.generics);
        self.writeln(&format!("trait{} TraitFrom{}{} {{", anno,
                              path_as_identifier(path), anno));
        self.indent();

        self.handle(blocks);

        self.writeln("fn render_trait_into(&self, writer: &mut std::fmt::Write) {");
        self.indent();
        self.handle(nodes);
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
                content.push(Node::Block(ws1, name, ws2));
            },
            _ => { content.push(n); },
        }
    }

    let mut gen = Generator::new();
    gen.path_based_name(ast, path);
    if !blocks.is_empty() {
        if let Some(extends) = base {
            if let Expr::StrLit(base_path) = extends {
                gen.trait_impl(ast, base_path, &blocks);
            }
        } else {
            gen.template_trait(ast, path, &blocks, &content);
            gen.trait_impl(ast, path, &Vec::new());
        }
        gen.trait_based_impl(ast);
    } else {
        gen.template_impl(ast, &content);
    }
    gen.result()
}
