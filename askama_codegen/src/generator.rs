use parser::Node;
use std::str;

pub fn generate(ctx_name: &str, tokens: &Vec<Node>) -> String {
    let mut code = String::new();
    code.push_str("impl askama::Template for ");
    code.push_str(ctx_name);
    code.push_str(" {\n");
    code.push_str("    fn render(&self) -> String {\n");
    code.push_str("        let mut buf = String::new();\n");
    for n in tokens {
        code.push_str("        buf.push_str(");
        code.push_str(&match n {
            &Node::Lit(val) => format!("{:#?}", str::from_utf8(val).unwrap()),
            &Node::Expr(val) => format!("&self.{}", str::from_utf8(val).unwrap()),
        });
        code.push_str(");\n");
    }
    code.push_str("        buf");
    code.push_str("    }\n");
    code.push_str("}\n\n");
    code
}
