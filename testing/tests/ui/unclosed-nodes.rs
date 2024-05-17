use askama::Template;

#[derive(Template)]
#[template(source = "{{ expr", ext = "txt")]
struct Expr1;

#[derive(Template)]
#[template(source = "{{ expr ", ext = "txt")]
struct Expr2;

#[derive(Template)]
#[template(source = "{{ expr -", ext = "txt")]
struct Expr3;

#[derive(Template)]
#[template(source = "{{ expr -}", ext = "txt")]
struct Expr4;

#[derive(Template)]
#[template(source = "{% let x", ext = "txt")]
struct Node1;

#[derive(Template)]
#[template(source = "{% let x ", ext = "txt")]
struct Node2;

#[derive(Template)]
#[template(source = "{% let x -", ext = "txt")]
struct Node3;

#[derive(Template)]
#[template(source = "{% let x -%", ext = "txt")]
struct Node4;

#[derive(Template)]
#[template(source = "{# comment", ext = "txt")]
struct Comment1;

#[derive(Template)]
#[template(source = "{# comment ", ext = "txt")]
struct Comment2;

#[derive(Template)]
#[template(source = "{# comment -", ext = "txt")]
struct Comment3;

#[derive(Template)]
#[template(source = "{# comment -#", ext = "txt")]
struct Comment4;

fn main() {}
