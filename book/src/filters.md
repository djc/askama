# Filters

Values such as those obtained from variables can be post-processed
using **filters**.
Filters are applied to values using the pipe symbol (`|`) and may
have optional extra arguments in parentheses.
Note that the pipe symbol must not be surrounded by spaces;
otherwise, it will be interpreted as the `BitOr` operator.
Filters can be chained, in which case the output from one filter
is passed to the next.

```
{{ "HELLO"|lower }}
```

Askama has a collection of built-in filters, documented below, but can also include custom filters. Additionally, the `json` and `yaml` filters are included in the built-in filters,
but are disabled by default. Enable them with Cargo features (see below for more information).

## Built-In Filters

### abs

Returns the absolute value.

```
{{ -2|abs }}
```

Output:

```
2
```

### capitalize

Capitalize a value. The first character will be uppercase, all others lowercase:

```
{{ "hello"|capitalize }}
```

Output:

```
Hello
```

### center

Centers the value in a field of a given width:

```
-{{ "a"|center(5) }}-
```

Output:
```
-  a  -
```

### escape | e

Escapes HTML characters in strings:

```
{{ "Escape <>&"|e }}
```

Output:

```
Escape &lt;&gt;&amp;
```

### filesizeformat

Returns adequate string representation (in KB, ..) of number of bytes:

```
{{ 1000|filesizeformat }}
```

Output:
```
1 KB
```

### format

Formats arguments according to the specified format.

The first argument to this filter must be a string literal (as in normal Rust).

All arguments are passed through to the `format!()` macro by the Askama code generator.

```
{{ "{:?}"|format(var) }}
```

### indent

Indent newlines with width spaces.

```
{{ "hello\nfoo\nbar"|indent(4) }}
```

Output:

```
hello
    foo
    bar
```

### join

Joins iterable into a string separated by provided argument.

```
array = &["foo", "bar", "bazz"]
```

```
{{ array|join(", ") }}
```

Output:

```
foo, bar, bazz
```

### linebreaks

Replaces line breaks in plain text with appropriate HTML.

A single newline becomes an HTML line break `<br>` and a new line followed by a blank line becomes a paragraph break `<p>`.

```
{{ "hello\nworld\n\nfrom\naskama"|linebreaks }}
```

Output:

```
<p>hello<br />world</p><p>from<br />askama</p>
```

### linebreaksbr

Converts all newlines in a piece of plain text to HTML line breaks.

```
{{ "hello\nworld\n\nfrom\naskama"|linebreaks }}
```

Output:

```
hello<br />world<br /><br />from<br />askama
```

### paragraphbreaks

A new line followed by a blank line becomes `<p>`, but, unlike `linebreaks`, single new lines are ignored and no `<br/>` tags are generated.

Consecutive double line breaks will be reduced down to a single paragraph break.

This is useful in contexts where changing single line breaks to line break tags would interfere with other HTML elements, such as lists and nested `<div>` tags.

```
{{ "hello\nworld\n\nfrom\n\n\n\naskama"|paragraphbreaks }}
```

Output:

```
<p>hello\nworld</p><p>from</p><p>askama</p>
```

### lower | lowercase

Converts to lowercase.

```
{{ "HELLO"|lower }}
```

Output:

```
hello
```

### safe

Marks a string (or other Display type) as safe. By default all strings are escaped according to the format.

```
{{ "<p>I'm Safe</p>"|safe }}
```

Output:

```
<p>I'm Safe</p>
```

### trim

Strip leading and trailing whitespace.

```
{{ " hello "|trim }}
```

Output:

```
hello
```

### truncate

Limit string length, appends '...' if truncated.


```
{{ "hello"|truncate(2) }}
```

Output:

```
he...
```

### upper | uppercase

Converts to uppercase.

```
{{ "hello"|upper }}
```

Output:

```
HELLO
```

### wordcount

Count the words in that string.

```
{{ "askama is sort of cool"|wordcount }}
```

Output:

```
5
```

## Custom Filters

To define your own filters, simply have a module named filters in scope of the context deriving a `Template` impl.

Note that in case of name collision, the built in filters take precedence.

```rust
#[derive(Template)]
#[template(source = "{{ s|myfilter }}", ext = "txt")]
struct MyFilterTemplate<'a> {
    s: &'a str,
}

mod filters {
    pub fn myfilter(s: &str) -> ::askama::Result<String> {
        Ok(s.replace("oo", "aa"))
    }
}

fn main() {
    let t = MyFilterTemplate { s: "foo" };
    assert_eq!(t.render().unwrap(), "faa");
}
```

## The `json` filter

Enabling the `serde-json` feature will enable the use of the `json` filter.
This will output formatted JSON for any value that implements the required
`Serialize` trait.

```jinja
{
  "foo": "{{ foo }}",
  "bar": {{ bar|json }}
}
```

For compatibility with Jinja, `tojson` can be used in place of `json`.

## The `yaml` filter

Enabling the `serde-yaml` feature will enable the use of the `yaml` filter.
This will output formatted JSON for any value that implements the required
`Serialize` trait.

```
{{ foo|yaml }}
```
