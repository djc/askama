use parser::Node;
use std::str;

struct Generator {
    buf: String,
}

impl Generator {
    fn new() -> Generator {
        Generator { buf: String::new() }
    }
    fn write(&mut self, s: &str) {
        self.buf.push_str(s);
    }
    fn result(self) -> String {
        self.buf
    }
}

pub fn generate(ctx_name: &str, tokens: &Vec<Node>) -> String {
    let mut gen = Generator::new();
    gen.write("impl askama::Template for ");
    gen.write(ctx_name);
    gen.write(" {\n");
    gen.write("    fn render(&self) -> String {\n");
    gen.write("        let mut buf = String::new();\n");
    for n in tokens {
        gen.write("        buf.push_str(");
        gen.write(&match n {
            &Node::Lit(val) => format!("{:#?}", str::from_utf8(val).unwrap()),
            &Node::Expr(val) => format!("&self.{}", str::from_utf8(val).unwrap()),
        });
        gen.write(");\n");
    }
    gen.write("        buf");
    gen.write("    }\n");
    gen.write("}\n\n");
    gen.result()
}
