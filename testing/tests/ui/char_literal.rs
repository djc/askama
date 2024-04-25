use askama::Template;

#[derive(Template)]
#[template(path = "char-literals/char-literal-1.txt")]
struct Err1;

#[derive(Template)]
#[template(path = "char-literals/char-literal-2.txt")]
struct Err2;

#[derive(Template)]
#[template(path = "char-literals/char-literal-3.txt")]
struct Err3;

#[derive(Template)]
#[template(path = "char-literals/char-literal-4.txt")]
struct Err4;

#[derive(Template)]
#[template(path = "char-literals/char-literal-5.txt")]
struct Err5;

#[derive(Template)]
#[template(path = "char-literals/char-literal-6.txt")]
struct Err6;

#[derive(Template)]
#[template(path = "char-literals/char-literal-7.txt")]
struct Err7;

#[derive(Template)]
#[template(source = "{% let s = 'aaa' %}", ext = "html")]
struct Err8;

fn main() {
}
