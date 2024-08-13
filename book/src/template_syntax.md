# Template Syntax

## Variables

Top-level template variables are defined by the template's context type.
You can use a dot (`.`) to access variable's attributes or methods.
Reading from variables is subject to the usual borrowing policies.
For example, `{{ name }}` will get the ``name`` field from the template
context,
while `{{ user.name }}` will get the ``name`` field of the ``user``
field from the template context.

## Using constants in templates

You can use constants defined in your Rust code. For example if you
have:

```rust
pub const MAX_NB_USERS: usize = 2;
```

defined in your crate root, you can then use it in your templates by
using ``crate::MAX_NB_USERS``:

```jinja
<p>The user limit is {{ crate::MAX_NB_USERS }}.</p>
{% set value = 4 %}
{% if value > crate::MAX_NB_USERS %}
    <p>{{ value }} is bigger than MAX_NB_USERS.</p>
{% else %}
    <p>{{ value }} is less than MAX_NB_USERS.</p>
{% endif %}
```

## Assignments

Inside code blocks, you can also declare variables or assign values
to variables.
Assignments can't be imported by other templates.

Assignments use the `let` tag:

```jinja
{% let name = user.name %}
{% let len = name.len() %}

{% let val -%}
{% if len == 0 -%}
  {% let val = "foo" -%}
{% else -%}
  {% let val = name -%}
{% endif -%}
{{ val }}
```

Like Rust, Askama also supports shadowing variables.

```jinja
{% let foo = "bar" %}
{{ foo }}

{% let foo = "baz" %}
{{ foo }}
```

For compatibility with Jinja, `set` can be used in place of `let`.

## Filters

Values such as those obtained from variables can be post-processed
using **filters**.
Filters are applied to values using the pipe symbol (`|`) and may
have optional extra arguments in parentheses.
Filters can be chained, in which case the output from one filter
is passed to the next.

For example, `{{ "{:?}"|format(name|escape) }}` will escape HTML
characters from the value obtained by accessing the `name` field,
and print the resulting string as a Rust literal.

The built-in filters are documented as part of the
[filters documentation](filters.md).

To define your own filters, simply have a module named `filters` in
scope of the context deriving a `Template` `impl`. Note that in case of
name collision, the built in filters take precedence.

## Filter blocks

You can apply a **filter** on a whole block at once using **filter
blocks**:

```text
{% filter lower %}
    {{ t }} / HELLO / {{ u }}
{% endfilter %}
```

The `lower` filter will be applied on the whole content.

Just like filters, you can combine them:

```text
{% filter lower|capitalize %}
    {{ t }} / HELLO / {{ u }}
{% endfilter %}
```

In this case, `lower` will be called and then `capitalize` will be
called on what `lower` returned.

## Whitespace control

Askama considers all tabs, spaces, newlines and carriage returns to be
whitespace. By default, it preserves all whitespace in template code,
except that a single trailing newline character is suppressed.
However, whitespace before and after expression and block delimiters
can be suppressed by writing a minus sign directly following a
start delimiter or leading into an end delimiter.

Here is an example:

```text
{% if foo %}
  {{- bar -}}
{% else if -%}
  nothing
{%- endif %}
```

This discards all whitespace inside the if/else block. If a literal
(any part of the template not surrounded by `{% %}` or `{{ }}`)
includes only whitespace, whitespace suppression on either side will
completely suppress that literal content.

If the whitespace default control is set to "suppress" and you want
to preserve whitespace characters on one side of a block or of an
expression, you need to use `+`. Example:

```text
<a href="/" {#+ #}
   class="something">text</a>
```

In the above example, one whitespace character is kept
between the `href` and the `class` attributes.

There is a third possibility. In case you want to suppress all whitespace
characters except one (`"minimize"`), you can use `~`:

```jinja
{% if something ~%}
Hello
{%~ endif %}
```

To be noted, if one of the trimmed characters is a newline, then the only
character remaining will be a newline.

Whitespace controls can also be defined by a
[configuration file](configuration.md) or in the derive macro.
These definitions follow the global-to-local preference:
1. Inline (`-`, `+`, `~`)
2. Derive (`#[template(whitespace = "suppress")]`)
3. Configuration (in `askama.toml`, `whitespace = "preserve"`)

Two inline whitespace controls may point to the same whitespace span.
In this case, they are resolved by the following preference.
1. Suppress (`-`)
2. Minimize (`~`)
3. Preserve (`+`)

## Functions

There are several ways that functions can be called within templates,
depending on where the function definition resides. These are:

- Template `struct` fields
- Static functions
- Struct/Trait implementations

### Template struct field

When the function is a field of the template struct, we can simply call it
by invoking the name of the field, followed by parentheses containing any
required arguments. For example, we can invoke the function `foo` for the
following `MyTemplate` struct:

```rust
#[derive(Template)]
#[template(source = "{{ foo(123) }}", ext = "txt")]
struct MyTemplate {
  foo: fn(u32) -> String,
}
```

However, since we'll need to define this function every time we create an
instance of `MyTemplate`, it's probably not the most ideal way to associate
some behaviour for our template.

### Static functions

When a function exists within the same Rust module as the template
definition, we can invoke it using the `self` path prefix, where `self`
represents the scope of the module in which the template struct resides.

For example, here we call the function `foo` by writing `self::foo(123)`
within the `MyTemplate` struct source:

```rust
fn foo(val: u32) -> String {
  format!("{}", val)
}

#[derive(Template)]
#[template(source = "{{ self::foo(123) }}", ext = "txt")]
struct MyTemplate;
```

This has the advantage of being able to share functionality across multiple
templates, without needing to expose the function publicly outside of its
module.

However, we are not limited to local functions defined within the same module.
We can call _any_ public function by specifying the full path to that function
within the template source. For example, given a utilities module such as:

```rust
// src/templates/utils/mod.rs

pub fn foo(val: u32) -> String {
  format!("{}", val)
}
```

Within our `MyTemplate` source, we can call the `foo` function by writing:

```rust
// src/templates/my_template.rs

#[derive(Template)]
#[template(source = "{{ crate::templates::utils::foo(123) }}", ext = "txt")]
struct MyTemplate;
```

### Struct / trait implementations

Finally, we can invoke functions that are implementation methods of our
template struct, by referencing `Self` (note the uppercase `S`) as the path,
before calling our function:

```rust
#[derive(Template)]
#[template(source = "{{ Self::foo(self, 123) }}", ext = "txt")]
struct MyTemplate {
  count: u32,
};

impl MyTemplate {
  fn foo(&self, val: u32) -> String {
    format!("{} is the count, {} is the value", self.count, val)
  }
}
```

If the implemented method requires a reference to the struct itself,
such as is demonstrated in the above example, we can pass `self`
(note the lowercase `s`) as the first argument.

Similarly, using the `Self` path, we can also call any method belonging
to a trait that has been implemented for our template struct:

```rust
trait Hello {
  fn greet(name: &str) -> String;
}

#[derive(Template)]
#[template(source = r#"{{ Self::greet("world") }}"#, ext = "txt")]
struct MyTemplate;

impl Hello for MyTemplate {
  fn greet(name: &str) -> String {
    format!("Hello {}", name)
  }
}
```

## Template inheritance

Template inheritance allows you to build a base template with common
elements that can be shared by all inheriting templates.
A base template defines **blocks** that child templates can override.

### Base template

```html
<!DOCTYPE html>
<html lang="en">
  <head>
    <title>{% block title %}{{ title }} - My Site{% endblock %}</title>
    {% block head %}{% endblock %}
  </head>
  <body>
    <div id="content">
      {% block content %}<p>Placeholder content</p>{% endblock %}
    </div>
  </body>
</html>
```

The `block` tags define three blocks that can be filled in by child
templates. The base template defines a default version of the block.
A base template must define one or more blocks in order to enable
inheritance. Blocks can only be specified at the top level of a template
or inside other blocks, not inside `if`/`else` branches or in `for`-loop
bodies.

It is also possible to use the name of the `block` in `endblock` (both in
declaration and use):

```html
{% block content %}<p>Placeholder content</p>{% endblock content %}
```

### Child template

Here's an example child template:

```html
{% extends "base.html" %}

{% block title %}Index{% endblock %}

{% block head %}
  <style>
  </style>
{% endblock %}

{% block content %}
  <h1>Index</h1>
  <p>Hello, world!</p>
  {% call super() %}
{% endblock %}
```

The `extends` tag tells the code generator that this template inherits
from another template. It will search for the base template relative to
itself before looking relative to the template base directory. It will
render the top-level content from the base template, and substitute
blocks from the base template with those from the child template. Inside
a block in a child template, the `super()` macro can be called to render
the parent block's contents.

Because top-level content from the child template is thus ignored, the `extends`
tag doesn't support whitespace control:

```html
{%- extends "base.html" +%}
```

The above code is rejected because we used `-` and `+`. For more information
about whitespace control, take a look [here](#whitespace-control).

### Block fragments

Additionally, a block can be rendered by itself. This can be useful when
you need to decompose your template for partial rendering, without
needing to extract the partial into a separate template or macro. This
can be done with the `block` parameter.

```rust
#[derive(Template)]
#[template(path = "...", block = "my_block")]
struct BlockFragment {
    name: String,
}
```

## HTML escaping

Askama by default escapes variables if it thinks it is rendering HTML
content. It infers the escaping context from the extension of template
filenames, escaping by default if the extension is one of `html`, `htm`,
or `xml`. When specifying a template as `source` in an attribute, the
`ext` attribute parameter must be used to specify a type. Additionally,
you can specify an escape mode explicitly for your template by setting
the `escape` attribute parameter value (to `none` or `html`).

Askama escapes `<`, `>`, `&`, `"`, and `'`, according to the
[OWASP escaping recommendations][owasp]. Use the `safe` filter to
prevent escaping for a single expression, or the `escape` (or `e`)
filter to escape a single expression in an unescaped context.

[owasp]: https://cheatsheetseries.owasp.org/cheatsheets/Cross_Site_Scripting_Prevention_Cheat_Sheet.html#output-encoding-for-html-contexts

```rust
#[derive(Template)]
#[template(source = "{{strvar}}")]
struct TestTemplate {
    strvar: String,
}

fn main() {
    let s = TestTemplate {
        strvar: "// my <html> is \"unsafe\" & should be 'escaped'".to_string(),
    };
    assert_eq!(
        s.render().unwrap(),
        "&#x2f;&#x2f; my &lt;html&gt; is &quot;unsafe&quot; &amp; \
         should be &#x27;escaped&#x27;"
    );
}
```

## Control structures

### For

Loop over each item in an iterator. For example:

```html
<h1>Users</h1>
<ul>
{% for user in users %}
  <li>{{ user.name|e }}</li>
{% endfor %}
</ul>
```

Inside for-loop blocks, some useful variables are accessible:

* *loop.index*: current loop iteration (starting from 1)
* *loop.index0*: current loop iteration (starting from 0)
* *loop.first*: whether this is the first iteration of the loop
* *loop.last*: whether this is the last iteration of the loop


```html
<h1>Users</h1>
<ul>
{% for user in users %}
   {% if loop.first %}
   <li>First: {{user.name}}</li>
   {% else %}
   <li>User#{{loop.index}}: {{user.name}}</li>
   {% endif %}
{% endfor %}
</ul>
```

### If

The `if` statement essentially mirrors Rust's [`if` expression],
and is used as you might expect:

```text
{% if users.len() == 0 %}
  No users
{% else if users.len() == 1 %}
  1 user
{% elif users.len() == 2 %}
  2 users
{% else %}
  {{ users.len() }} users
{% endif %}
```

[`if` expression]: https://doc.rust-lang.org/reference/expressions/if-expr.html#if-expressions

#### If Let

Additionally, `if let` statements are also supported and similarly
mirror Rust's [`if let` expressions]:

```text
{% if let Some(user) = user %}
  {{ user.name }}
{% else %}
  No user
{% endif %}
```

[`if let` expressions]: https://doc.rust-lang.org/reference/expressions/if-expr.html#if-let-expressions

### Match

In order to deal with Rust `enum`s in a type-safe way, templates support
match blocks from version 0.6. Here is a simple example showing how to
expand an `Option`:

```text
{% match item %}
  {% when Some with ("foo") %}
    Found literal foo
  {% when Some with (val) %}
    Found {{ val }}
  {% when None %}
{% endmatch %}
```

That is, a `match` block can optionally contain some whitespace (but
no other literal content), followed by a number of `when` blocks
and an optional `else` block. Each `when` block must name a list of
matches (`(val)`), optionally introduced with a variant name. The
`else` block is equivalent to matching on `_` (matching anything).

Struct-like enum variants are supported from version 0.8, with the list
of matches surrounded by curly braces instead (`{ field }`).  New names
for the fields can be specified after a colon in the list of matches
(`{ field: val }`).

### Include

The *include* statement lets you split large or repetitive blocks into
separate template files. Included templates get full access to the context
in which they're used, including local variables like those from loops:

```text
{% for i in iter %}
  {% include "item.html" %}
{% endfor %}
```

```text
* Item: {{ i }}
```

The path to include must be a string literal, so that it is known at
compile time. Askama will try to find the specified template relative
to the including template's path before falling back to the absolute
template path. Use `include` within the branches of an `if`/`else`
block to use includes more dynamically.

## Expressions

Askama supports string literals (`"foo"`) and integer literals (`1`).
It supports almost all binary operators that Rust supports,
including arithmetic, comparison and logic operators.
The parser applies the same precedence order as the Rust compiler.
Expressions can be grouped using parentheses.
The HTML special characters `&`, `<` and `>` will be replaced with their
character entities unless the `escape` mode is disabled for a template.
Methods can be called on variables that are in scope, including `self`.

```
{{ 3 * 4 / 2 }}
{{ 26 / 2 % 7 }}
{{ 3 % 2 * 6 }}
{{ 1 * 2 + 4 }}
{{ 11 - 15 / 3 }}
{{ 4 + 5 % 3 }}
{{ 4 | 2 + 5 & 2 }}
```

**Warning**: if the result of an expression (a `{{ }}` block) is
equivalent to `self`, this can result in a stack overflow from infinite
recursion. This is because the `Display` implementation for that expression
will in turn evaluate the expression and yield `self` again.


## Templates in templates

Using expressions, it is possible to delegate rendering part of a template to another template.
This makes it possible to inject modular template sections into other templates and facilitates
testing and reuse.

```rust
use askama::Template;
#[derive(Template)]
#[template(source = "Section 1: {{ s1 }}", ext = "txt")]
struct RenderInPlace<'a> {
   s1: SectionOne<'a>
}

#[derive(Template)]
#[template(source = "A={{ a }}\nB={{ b }}", ext = "txt")]
struct SectionOne<'a> {
   a: &'a str,
   b: &'a str,
}

let t = RenderInPlace { s1: SectionOne { a: "a", b: "b" } };
assert_eq!(t.render().unwrap(), "Section 1: A=a\nB=b")
```

See the example
[render in place](https://github.com/djc/askama/blob/main/testing/tests/render_in_place.rs)
using a vector of templates in a for block.

## Comments

Askama supports block comments delimited by `{#` and `#}`.

```jinja
{# A Comment #}
```

Like Rust, Askama also supports nested block comments.

```jinja
{#
A Comment
{# A nested comment #}
#}
```

## Recursive Structures

Recursive implementations should preferably use a custom iterator and
use a plain loop. If that is not doable, call `.render()`
directly by using an expression as shown below.
Including self does not work, see #105 and #220 .

```rust
use askama::Template;

#[derive(Template)]
#[template(source = r#"
//! {% for item in children %}
   {{ item }}
{% endfor %}
"#, ext = "html", escape = "none")]
struct Item<'a> {
    name: &'a str,
    children: &'a [Item<'a>],
}
```

## Macros

You can define macros within your template by using `{% macro name(args) %}`, ending with `{% endmacro %}`.

You can then call it with `{% call name(args) %}`:

```jinja
{% macro heading(arg) %}

<h1>{{arg}}</h1>

{% endmacro %}

{% call heading(s) %}
```

You can place macros in a separate file and use them in your templates by using `{% import %}`:

```jinja
{%- import "macro.html" as scope -%}

{% call scope::heading(s) %}
```

You can optionally specify the name of the macro in `endmacro`:

```jinja
{% macro heading(arg) %}<p>{{arg}}</p>{% endmacro heading %}
```

You can also specify arguments by their name (as defined in the macro):

```jinja
{% macro heading(arg, bold) %}

<h1>{{arg}} <b>{{bold}}</b></h1>

{% endmacro %}

{% call heading(bold="something", arg="title") %}
```

You can use whitespace characters around `=`:

```jinja
{% call heading(bold = "something", arg = "title") %}
```

You can mix named and non-named arguments when calling a macro:

```
{% call heading("title", bold="something") %}
```

However please note than named arguments must always come **last**.

Another thing to note, if a named argument is referring to an argument that would
be used for a non-named argument, it will error:

```jinja
{% macro heading(arg1, arg2, arg3, arg4) %}
{% endmacro %}

{% call heading("something", "b", arg4="ah", arg2="title") %}
```

In here it's invalid because `arg2` is the second argument and would be used by
`"b"`. So either you replace `"b"` with `arg3="b"` or you pass `"title"` before:

```jinja
{% call heading("something", arg3="b", arg4="ah", arg2="title") %}
{# Equivalent of: #}
{% call heading("something", "title", "b", arg4="ah") %}
```

## Calling Rust macros

It is possible to call rust macros directly in your templates:

```jinja
{% let s = format!("{}", 12) %}
```

One important thing to note is that contrary to the rest of the expressions,
Askama cannot know if a token given to a macro is a variable or something
else, so it will always default to generate it "as is". So if you have:

```rust
macro_rules! test_macro{
    ($entity:expr) => {
        println!("{:?}", &$entity);
    }
}

#[derive(Template)]
#[template(source = "{{ test_macro!(entity) }}", ext = "txt")]
struct TestTemplate<'a> {
    entity: &'a str,
}
```

It will not compile, telling you it doesn't know `entity`. It didn't infer
that `entity` was a field of the current type unlike usual. You can go
around this limitation by binding your field's value into a variable:

```jinja
{% let entity = entity %}
{{ test_macro!(entity) }}
```
