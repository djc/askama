# Askama

[![Latest version](https://img.shields.io/crates/v/askama.svg)](https://crates.io/crates/askama)
[![Build status](https://api.travis-ci.org/djc/askama.svg?branch=master)](https://travis-ci.org/djc/askama)
[![Code coverage](https://codecov.io/gh/djc/askama/branch/master/graph/badge.svg)](https://codecov.io/gh/djc/askama)

Askama implements a template rendering engine based on Jinja.
It generates Rust code from your templates at compile time
based on a user-defined `struct` to hold the template's context.
See below for an example, or read [the documentation][docs].

**"Pretty exciting. I would love to use this already."** --
[Armin Ronacher][mitsuhiko], creator of Jinja

Currently implemented features:

* Generates fully type-safe Rust code from your templates
* Template inheritance
* Basic loops and if/else if/else statements
* Whitespace suppressing with '-' markers
* Some built-in filters
* Works on stable Rust

Askama is in heavy development, so it currently has some limitations:

* Only a small number of built-in template filters have been implemented
* User-defined template filters are not supported yet
* Debugging template problems can be tricky

All feedback welcome. Feel free to file bugs, requests for documentation and
any other feedback to the [issue tracker][issues] or [tweet me][twitter].

[docs]: https://docs.rs/askama
[mitsuhiko]: http://lucumr.pocoo.org/
[issues]: https://github.com/djc/askama/issues
[twitter]: https://twitter.com/djco/


How to get started
------------------

First, add the following to your crate's `Cargo.toml`:

```toml
# in section [package]
build = "build.rs"

# in section [dependencies]
askama = "0.1"
askama_derive = "0.1"

# in section [build-dependencies]
askama = "0.1"
```

Because Askama will generate Rust code from your template files,
the crate will need to be recompiled when your templates change.
This is supported by adding a build script, `build.rs`, to your crate.
It needs askama as a build dependency:

```rust
extern crate askama;

fn main() {
    askama::rerun_if_templates_changed();
}
```

Now create a directory called `templates` in your crate root.
In it, create a file called `hello.html`, containing the following:

```
Hello, {{ name }}!
```

In any Rust file inside your crate, add the following:

```rust
#[macro_use]
extern crate askama; // for the Template trait and custom derive macro

use askama::Template; // bring trait in scope

#[derive(Template)] // this will generate the code...
#[template(path = "hello.html")] // using the template in this path, relative
                                 // to the templates dir in the crate root
struct HelloTemplate<'a> { // the name of the struct can be anything
    name: &'a str, // the field name should match the variable name
                   // in your template
}
   
fn main() {
    let hello = HelloTemplate { name: "world" }; // instantiate your struct
    println!("{}", hello.render()); // then render it.
}
```

You should now be able to compile and run this code.

Review the [test cases] for more examples.

[test cases]: https://github.com/djc/askama/tree/master/testing


Debugging and troubleshooting
-----------------------------

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

The parse tree looks like this for the example template:

```
[Lit("", "Hello,", " "), Expr(WS(false, false), Var("name")),
Lit("", "!", "\n")]
```

The generated code looks like this:

```rust
#[allow(dead_code, non_camel_case_types)]
impl<'a> askama::Template for HelloTemplate<'a> {
    fn render_to(&self, writer: &mut std::fmt::Write) {
        writer.write_str("Hello,").unwrap();
        writer.write_str(" ").unwrap();
        writer.write_fmt(format_args!("{}", self.name)).unwrap();
        writer.write_str("!").unwrap();
        writer.write_str("\n").unwrap();
    }
}
```
