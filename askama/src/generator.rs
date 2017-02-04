use parser::{Conds, Expr, Node, Nodes, Target};
use std::str;
use std::collections::HashSet;
use syn;

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

    fn annotations(&self, generics: &syn::Generics) -> String {
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

    fn init(&mut self, name: &str, generics: &syn::Generics) {
        self.write("impl");
        let anno = self.annotations(generics);
        self.write(&anno);
        self.write(" askama::Template for ");
        self.write(name);
        self.write(&anno);
        self.writeln(" {");

        self.indent();
        self.writeln("fn render(&self) -> String {");
        self.indent();
        self.writeln("let mut buf = String::new();");
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

    fn visit_var(&mut self, s: &[u8]) {
        let s = str::from_utf8(s).unwrap();
        if self.locals.contains(s) {
            self.write(&format!("{}", s));
        } else {
            self.write(&format!("self.{}", s));
        }
    }

    fn visit_filter(&mut self, name: &str, val: &Expr) {
        self.write(&format!("askama::filters::{}(&", name));
        self.visit_expr(val);
        self.write(")");
    }

    fn visit_expr(&mut self, expr: &Expr) {
        match expr {
            &Expr::Var(s) => self.visit_var(s),
            &Expr::Filter(name, ref val) => self.visit_filter(name, &val),
        }
    }

    fn visit_target_single(&mut self, name: &[u8]) -> Vec<String> {
        vec![str::from_utf8(name).unwrap().to_string()]
    }

    fn visit_target(&mut self, target: &Target) -> Vec<String> {
        match target {
            &Target::Name(s) => { self.visit_target_single(s) },
        }
    }

    fn write_lit(&mut self, s: &[u8]) {
        self.write("buf.push_str(");
        self.write(&format!("{:#?}", str::from_utf8(s).unwrap()));
        self.writeln(");");
    }

    fn write_expr(&mut self, s: &Expr) {
        self.write("std::fmt::Write::write_fmt(&mut buf, format_args!(\"{}\", ");
        self.visit_expr(s);
        self.writeln(")).unwrap();");
    }

    fn write_cond(&mut self, conds: &Conds) {
        for (i, &(ref cond, ref nodes)) in conds.iter().enumerate() {
            match cond {
                &Some(ref expr) => {
                    if i == 0 {
                        self.write("if ");
                    } else {
                        self.write("} else if ");
                    }
                    self.visit_expr(expr);
                },
                &None => { self.writeln("} else"); },
            }
            self.writeln(" {");
            self.indent();
            self.handle(nodes);
            self.dedent();
        }
        self.writeln("}");
    }

    fn write_loop(&mut self, var: &Target, iter: &Expr, body: &Nodes) {

        self.write("for ");
        let targets = self.visit_target(var);
        for name in &targets {
            self.locals.insert(name.clone());
            self.write(&format!("{}", name));
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

    fn handle(&mut self, tokens: &Vec<Node>) {
        for n in tokens {
            match n {
                &Node::Lit(val) => { self.write_lit(val); },
                &Node::Expr(ref val) => { self.write_expr(&val); },
                &Node::Cond(ref conds) => { self.write_cond(&conds); },
                &Node::Loop(ref var, ref iter, ref body) => {
                    self.write_loop(&var, &iter, &body);
                },
            }
        }
    }

    fn finalize(&mut self) {
        self.writeln("buf");
        self.dedent();
        self.writeln("}");
        self.dedent();
        self.writeln("}");
    }

    fn result(self) -> String {
        self.buf
    }

}

pub fn generate(ast: &syn::DeriveInput, tokens: &Vec<Node>) -> String {
    let mut gen = Generator::new();
    gen.init(ast.ident.as_ref(), &ast.generics);
    gen.handle(tokens);
    gen.finalize();
    gen.result()
}
