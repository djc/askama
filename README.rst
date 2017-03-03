Askama
======

.. image:: https://travis-ci.org/djc/askama.svg?branch=master
   :target: https://travis-ci.org/djc/askama

Askama implements a template rendering engine based on Jinja.
It generates Rust code from your templates at compile time
based on a user-defined ``struct`` to hold the template's context.
See below for an example.

Currently implemented features:

* Generates fully type-safe Rust code from your templates
* Template inheritance
* Basic loops and if/else if/else statements
* Whitespace suppressing with '-' markers
* Some built-in filters

Askama is in heavy development, so it currently has some limitations:

* Only a small number of built-in template filters have been implemented
* User-defined template filters are not supported yet
* Not a lot of documentation has been written
* Debugging template problems is not always straightforward

All feedback welcome. Feel free to file bugs, requests for documentation and
any other feedback to the `issue tracker`_ or `tweet me`_.

.. _issue tracker: https://github.com/djc/askama/issues
.. _tweet me: https://twitter.com/djco/


How to get started
------------------

First, add the following to your crate's ``Cargo.toml``:

.. code-block::
   
   // in section [package]
   build = "build.rs"
   
   // in section [dependencies]
   askama = "0.1"
   askama_derive = "0.1"
   
   // in section [build-dependencies]
   askama = "0.1"

Custom derive macros can not be exported together with other items,
so you have to depend on a separate crate for it.
Because Askama will generate Rust code from your template files,
the crate will need to be recompiled when your templates change.
This is supported with a build script, ``build.rs``,
which needs askama as a build dependency:

.. code-block:: rust
   
   extern crate askama;
   
   fn main() {
       askama::rerun_if_templates_changed();
   }

Now create a directory called ``templates`` in your crate root.
In it, create a file called ``hello.html``, containing the following:

.. code-block:: jinja
   
   Hello, {{ name }}!

In any Rust file inside your crate, add the following:

.. code-block:: rust
   
   #[macro_use]
   extern crate askama_derive; // for the custom derive implementation
   extern crate askama; // for the Template trait
   
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

You should now be able to compile and run this code.

Review the `test cases`_ for more examples.

.. _test cases: https://github.com/djc/askama/tree/master/testing


Debugging and troubleshooting
-----------------------------

You can debug your the parse tree for a template and the generated code by
changing the ``template`` attribute item list for the template struct:

.. code-block:: rust

   #[derive(Template)]
   #[template(path = "hello.html", print = "all")]
   struct HelloTemplate<'a> { ... }

The ``print`` key can take one of four values:

* ``none`` (the default value)
* ``ast`` (print the parse tree)
* ``code`` (print the generated code)
* ``all`` (print both parse tree and code)

The parse tree looks like this for the example template:

.. code-block::

   [Lit("", "Hello,", " "), Expr(WS(false, false), Var("name")),
   Lit("", "!", "\n")]

The generated code looks like this:

.. code-block:: rust
   
   #[allow(dead_code, non_camel_case_types)]
   type TemplateFromhello2ehtml<'a> = HelloTemplate<'a>;
   impl<'a> askama::Template for HelloTemplate<'a> {
       fn render_to(&self, writer: &mut std::fmt::Write) {
           writer.write_str("Hello,").unwrap();
           writer.write_str(" ").unwrap();
           writer.write_fmt(format_args!("{}", self.name)).unwrap();
           writer.write_str("!").unwrap();
           writer.write_str("\n").unwrap();
       }
   }
