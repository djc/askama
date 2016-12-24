use std::str;

pub fn generate(ctx_name: &str, tokens: &Vec<&[u8]>) -> String {
    let mut code = String::new();
    code.push_str("impl askama::Template for ");
    code.push_str(ctx_name);
    code.push_str(" {\n");
    code.push_str("    fn render(&self) -> String {\n");
    code.push_str("        let mut buf = String::new();\n");
    code.push_str("        buf.push_str(\"");
    code.push_str(str::from_utf8(tokens[0]).unwrap());
    code.push_str("\");\n");
    code.push_str("        buf.push_str(&self.");
    code.push_str(str::from_utf8(tokens[1]).unwrap());
    code.push_str(");\n");
    code.push_str("        buf.push_str(\"");
    code.push_str(str::from_utf8(tokens[2]).unwrap());
    code.push_str("\");\n");
    code.push_str("        buf");
    code.push_str("    }\n");
    code.push_str("}\n\n");
    code
}
