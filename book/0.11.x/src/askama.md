# Askama

[![Documentation](https://docs.rs/askama/badge.svg)](https://docs.rs/askama/)
[![Chat](https://badges.gitter.im/gitterHQ/gitter.svg)](https://gitter.im/djc/askama)

Askama implements a template rendering engine based on Jinja.
It generates Rust code from your templates at compile time
based on a user-defined `struct` to hold the template's context.
See below for an example, or read [the book][docs].

**"Pretty exciting. I would love to use this already."** --
[Armin Ronacher][mitsuhiko], creator of Jinja

All feedback welcome. Feel free to file bugs, requests for documentation and
any other feedback to the [issue tracker][issues] or [tweet me][twitter].

Askama was created by and is maintained by Dirkjan Ochtman. If you are in a
position to support ongoing maintenance and further development or use it
in a for-profit context, please consider supporting my open source work on
[Patreon][patreon].

### Feature highlights

* Construct templates using a familiar, easy-to-use syntax
* Benefit from the safety provided by Rust's type system
* Template code is compiled into your crate for [optimal performance][benchmarks]
* Optional built-in support for Actix, Gotham, Iron, Rocket and warp web frameworks
* Debugging features to assist you in template development
* Templates must be valid UTF-8 and produce UTF-8 when rendered
* Works on stable Rust

### Supported in templates

* Template inheritance
* Loops, if/else statements and include support
* Macro support
* Variables (no mutability allowed)
* Some built-in filters, and the ability to use your own
* Whitespace suppressing with '-' markers
* Opt-out HTML escaping
* Syntax customization

[docs]: https://djc.github.io/askama/
[fafhrd91]: https://github.com/fafhrd91
[mitsuhiko]: http://lucumr.pocoo.org/
[issues]: https://github.com/djc/askama/issues
[twitter]: https://twitter.com/djco/
[patreon]: https://www.patreon.com/dochtman
[benchmarks]: https://github.com/djc/template-benchmarks-rs
