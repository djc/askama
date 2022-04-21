# Configuration

At compile time, Askama will read optional configuration values from
`askama.toml` in the crate root (the directory where `Cargo.toml` can
be found). Currently, this covers the directories to search for templates,
custom syntax configuration and escaper configuration.

This example file demonstrates the default configuration:

```toml
[general]
# Directories to search for templates, relative to the crate root.
dirs = ["templates"]
# Unless you add a `-` in a block, whitespace characters won't be trimmed.
whitespace = "preserve"
```

In the default configuration, you can use the `-` operator to indicate that
whitespace should be suppressed before or after a block. For example:

```jinja
<div>


{%- if something %}
Hello
{% endif %}
```

In the template above, only the whitespace between `<div>` and `{%-` will be
suppressed. If you set `whitespace` to `"suppress"`, whitespace characters before
and after each block will be suppressed by default. To preserve the whitespace
characters, you can use the `+` operator:

```jinja
{% if something +%}
Hello
{%+ endif %}
```

In this example, `Hello` will be surrounded with newline characters.

There is a third possibility: in case you want to suppress all whitespace
characters except one, you can use `~`:

```jinja
{% if something ~%}
Hello
{%~ endif %}
```

To be noted, if one of the trimmed characters is a newline, then the only
character remaining will be a newline.

If you want this to be the default behaviour, you can set `whitespace` to
`"minimize"`.

Here is an example that defines two custom syntaxes:

```toml
[general]
default_syntax = "foo"

[[syntax]]
name = "foo"
block_start = "%{"
comment_start = "#{"
expr_end = "^^"

[[syntax]]
name = "bar"
block_start = "%%"
block_end = "%%"
comment_start = "%#"
expr_start = "%{"
```

A syntax block consists of at least the attribute `name` which uniquely
names this syntax in the project.

The following keys can currently be used to customize template syntax:

* `block_start`, defaults to `{%`
* `block_end`, defaults to `%}`
* `comment_start`, defaults to `{#`
* `comment_end`, defaults to `#}`
* `expr_start`, defaults to `{{`
* `expr_end`, defaults to `}}`

Values must be 2 characters long and start delimiters must all start with the same
character. If a key is omitted, the value from the default syntax is used.

Here is an example of a custom escaper:

```toml
[[escaper]]
path = "::tex_escape::Tex"
extensions = ["tex"]
```

An escaper block consists of the attributes `path` and `name`. `path`
contains a Rust identifier that must be in scope for templates using this
escaper. `extensions` defines a list of file extensions that will trigger
the use of that escaper. Extensions are matched in order, starting with the
first escaper configured and ending with the default escapers for HTML
(extensions `html`, `htm`, `xml`, `j2`, `jinja`, `jinja2`) and plain text
(no escaping; `md`, `yml`, `none`, `txt`, and the empty string). Note that
this means you can also define other escapers that match different extensions
to the same escaper.
