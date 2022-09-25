# Debugging and Troubleshooting

You can view the parse tree for a template as well as the generated code by
changing the `template` attribute item list for the template struct:

```rust
#[derive(Template)]
#[template(path = "hello.html", print = "all")]
struct HelloTemplate<'a> { ... }
```

The `print` key can take one of four values:

* `none` (the default value)
* `ast` (print the parse tree)
* `code` (print the generated code)
* `all` (print both parse tree and code)

The resulting output will be printed to `stderr` during the compilation process.

The parse tree looks like this for the example template:

```
[Lit("", "Hello,", " "), Expr(WS(false, false), Var("name")),
Lit("", "!", "\n")]
```

The generated code looks like this:

```rust
impl < 'a > ::askama::Template for HelloTemplate< 'a > {
    fn render_into(&self, writer: &mut ::std::fmt::Write) -> ::askama::Result<()> {
        write!(
            writer,
            "Hello, {expr0}!",
            expr0 = &::askama::MarkupDisplay::from(&self.name),
        )?;
        Ok(())
    }
    fn extension() -> Option<&'static str> {
        Some("html")
    }
}
impl < 'a > ::std::fmt::Display for HelloTemplate< 'a > {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::askama::Template::render_into(self, f).map_err(|_| ::std::fmt::Error {})
    }
}
```
