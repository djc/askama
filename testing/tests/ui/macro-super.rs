use askama::Template;

#[derive(Template)]
#[template(source = "{%- macro super() -%}{%- endmacro -%}", ext = "html")]
struct MacroSuper;

fn main() {
}
