# Template Syntax

## Variables

Top-level template variables are defined by the template's context type.
You can use a dot (`.`) to access variable's attributes or methods.
Reading from variables is subject to the usual borrowing policies.
For example, `{{ name }}` will get the ``name`` field from the template
context,
while `{{ user.name }}` will get the ``name`` field of the ``user``
field from the template context.

## Assignments

Inside code blocks, you can also declare variables or assign values
to variables.
Assignments can't be imported by other templates.

Assignments use the let tag:

```text
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
      {% block content %}{% endblock %}
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
{% endblock %}
```

The `extends` tag tells the code generator that this template inherits
from another template. It will search for the base template relative to
itself before looking relative to the template base directory. It will
render the top-level content from the base template, and substitute
blocks from the base template with those from the child template. Inside
a block in a child template, the `super()` macro can be called to render
the parent block's contents.

## HTML escaping

Askama by default escapes variables if it thinks it is rendering HTML
content. It infers the escaping context from the extension of template
filenames, escaping by default if the extension is one of `html`, `htm`,
or `xml`. When specifying a template as `source` in an attribute, the
`ext` attribute parameter must be used to specify a type. Additionally,
you can specify an escape mode explicitly for your template by setting
the `escape` attribute parameter value (to `none` or `html`).

Askama escapes `<`, `>`, `&`, `"`, `'`, `\` and `/`, according to the
[OWASP escaping recommendations][owasp]. Use the `safe` filter to
prevent escaping for a single expression, or the `escape` (or `e`)
filter to escape a single expression in an unescaped context.

[owasp]: https://www.owasp.org/index.php/XSS_(Cross_Site_Scripting)_Prevention_Cheat_Sheet#RULE_.231_-_HTML_Escape_Before_Inserting_Untrusted_Data_into_HTML_Element_Content

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

The *if* statement is used as you might expect:

```text
{% if users.len() == 0 %}
  No users
{% else if users.len() == 1 %}
  1 user
{% else %}
  {{ users.len() }} users
{% endif %}
```

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
#[template(source = "Section 1: {{ s1.render().unwrap() }}", ext = "txt")]
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
[render in place](https://github.com/djc/askama/blob/master/testing/tests/render_in_place.rs)
using a vector of templates in a for block.

## Comments

Askama supports block comments delimited by `{#` and `#}`.

```
{# A Comment #}
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
   {{ item.render().unwrap() }}
{% endfor %}
"#, ext = "html", escape = "none")]
struct Item<'a> {
    name: &'a str,
    children: &'a [Item<'a>],
}
```

## Macros

You can define macros within your template by using `{% macro name(args) %}`, ending with `{% endmacro %}`

You can then call it later with `{% call name(args) %}`

```
{% macro heading(arg) %}

<h1>{{arg}}</h1>

{% endmacro %}

{% call heading(s) %}
```
