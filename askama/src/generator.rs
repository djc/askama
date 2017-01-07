use parser::{Conds, Expr, Node};
use std::str;

struct Generator {
    buf: String,
    indent: u8,
    start: bool,
}

impl Generator {

    fn new() -> Generator {
        Generator { buf: String::new(), indent: 0, start: true }
    }

    fn init(&mut self, name: &str) {
        self.write("impl askama::Template for ");
        self.write(name);
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
        self.write(&format!("self.{}", str::from_utf8(s).unwrap()));
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

    fn handle(&mut self, tokens: &Vec<Node>) {
        for n in tokens {
            match n {
                &Node::Lit(val) => { self.write_lit(val); },
                &Node::Expr(ref val) => { self.write_expr(&val); },
                &Node::Cond(ref conds) => { self.write_cond(&conds); },
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

pub fn generate(ctx_name: &str, tokens: &Vec<Node>) -> String {
    let mut gen = Generator::new();
    gen.init(ctx_name);
    gen.handle(tokens);
    gen.finalize();
    gen.result()
}
