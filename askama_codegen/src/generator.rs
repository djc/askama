use parser::Node;
use std::str;

struct Generator {
    buf: String,
}

impl Generator {

    fn new() -> Generator {
        Generator { buf: String::new() }
    }

    fn init(&mut self, name: &str) {
        self.write("impl askama::Template for ");
        self.write(name);
        self.write(" {\n");
        self.write("    fn render(&self) -> String {\n");
        self.write("        let mut buf = String::new();\n");
    }

    fn write(&mut self, s: &str) {
        self.buf.push_str(s);
    }

    fn visit_lit(&mut self, s: &[u8]) {
        self.write("        buf.push_str(");
        self.write(&format!("{:#?}", str::from_utf8(s).unwrap()));
        self.write(");\n");
    }

    fn visit_expr(&mut self, s: &[u8]) {
        self.write("        buf.push_str(");
        self.write(&format!("&self.{}", str::from_utf8(s).unwrap()));
        self.write(");\n");
    }

    fn handle(&mut self, tokens: &Vec<Node>) {
        for n in tokens {
            match n {
                &Node::Lit(val) => { self.visit_lit(val); },
                &Node::Expr(val) => { self.visit_expr(val); },
            }
        }
    }

    fn finalize(&mut self) {
        self.write("        buf");
        self.write("    }\n");
        self.write("}\n\n");
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
