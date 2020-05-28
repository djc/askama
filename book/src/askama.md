# Askama

[![Documentation](https://docs.rs/askama/badge.svg)](https://docs.rs/askama/)
[![Latest version](https://img.shields.io/crates/v/askama.svg)](https://crates.io/crates/askama)
[![Build Status](https://github.com/djc/askama/workflows/CI/badge.svg)](https://github.com/djc/askama/actions?query=workflow%3ACI)
[![Chat](https://badges.gitter.im/gitterHQ/gitter.svg)](https://gitter.im/djc/askama)

Askama implements a template rendering engine based on Jinja.
It generates Rust code from your templates at compile time
based on a user-defined `struct` to hold the template's context.
See below for an example, or read [the documentation][docs].

**"I use Askama for actix's TechEmpower benchmarks."** --
[Nikolay Kim][fafhrd91], creator of actix-web

**"Pretty exciting. I would love to use this already."** --
[Armin Ronacher][mitsuhiko], creator of Jinja

All feedback welcome. Feel free to file bugs, requests for documentation and
any other feedback to the [issue tracker][issues] or [tweet me][twitter].
Many thanks to [David Tolnay][dtolnay] for his support in improving Askama.

Askama was created by and is maintained by Dirkjan Ochtman. If you are in a
position to support ongoing maintenance and further development or use it
in a for-profit context, please consider supporting my open source work on
[Patreon][patreon].

### Feature highlights

* Construct templates using a familiar, easy-to-use syntax
* Template code is compiled into your crate for [optimal performance][benchmarks]
* Benefit from the safety provided by Rust's type system
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

[docs]: https://docs.rs/askama
[fafhrd91]: https://github.com/fafhrd91
[mitsuhiko]: http://lucumr.pocoo.org/
[issues]: https://github.com/djc/askama/issues
[twitter]: https://twitter.com/djco/
[dtolnay]: https://github.com/dtolnay
[patreon]: https://www.patreon.com/dochtman
[benchmarks]: https://github.com/djc/template-benchmarks-rs
