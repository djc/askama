# Template Expansion

This chapter will explain how the different parts of the templates are
translated into Rust code.

⚠️ Please note that the generated code might change in the future so the
following examples might not be up-to-date.

## Basic explanations

When you add `#[derive(Template)]` and `#[template(...)]` on your type, the
`Template` derive proc-macro will then generate an implementation of the
`askama::Template` trait which will be a Rust version of the template.

It will also implement the `std::fmt::Display` trait on your type which will
internally call the `askama::Template` trait.

Let's take a small example:

```rust
#[derive(Template)]
#[template(source = "{% set x = 12 %}", ext = "html")]
struct Mine;
```

will generate:

```rust
impl ::askama::Template for YourType {
    fn render_into(
        &self,
        writer: &mut (impl ::std::fmt::Write + ?Sized),
    ) -> ::askama::Result<()> {
        let x = 12;
        ::askama::Result::Ok(())
    }
    const EXTENSION: ::std::option::Option<&'static ::std::primitive::str> = Some(
        "html",
    );
    const SIZE_HINT: ::std::primitive::usize = 0;
    const MIME_TYPE: &'static ::std::primitive::str = "text/html; charset=utf-8";
}

impl ::std::fmt::Display for YourType {
    #[inline]
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::askama::Template::render_into(self, f).map_err(|_| ::std::fmt::Error {})
    }
}
```

For simplicity, we will only keep the content of the `askama::Template::render_into`
function from now on.

## Text content

If you have "text content" (for example HTML) in your template:

```html
<h1>{{ title }}</h1>
```

It will generate it like this:

```rust
writer
    .write_fmt(
        format_args!(
            "<h1>{0}</h1>",
            &::askama::MarkupDisplay::new_unsafe(&(self.title), ::askama::Html),
        ),
    )?;
::askama::Result::Ok(())
```

About `MarkupDisplay`: we need to use this type in order to prevent generating
invalid HTML. Let's take an example: if `title` is `"<a>"` and we display it as
is, in the generated HTML, you won't see `<a>` but instead a new HTML element
will be created. To prevent this, we need to escape some characters.

In this example, `<a>` will become `&lt;a&gt;`. And this is why there is the
`safe` builtin filter, in case you want it to be displayed as is.

## Variables

### Variables creation

If you create a variable in your template, it will be created in the generated
Rust code as well. For example:

```jinja
{% set x = 12 %}
{% let y = x + 1 %}
```

will generate:

```rust
let x = 12;
let y = x + 1;
::askama::Result::Ok(())
```

### Variables usage

By default, variables will reference a field from the type on which the `askama::Template`
trait is implemented:

```jinja
{{ y }}
```

This template will expand as follows:

```rust
writer
    .write_fmt(
        format_args!(
            "{0}",
            &::askama::MarkupDisplay::new_unsafe(&(self.y), ::askama::Html),
        ),
    )?;
::askama::Result::Ok(())
```

This is why if the variable is undefined, it won't work with Askama and why
we can't check if a variable is defined or not.

You can still access constants and statics by using paths. Let's say you have in
your Rust code:

```rust
const FOO: u32 = 0;
```

Then you can use them in your template by referring to them with a path:

```jinja
{{ crate::FOO }}{{ super::FOO }}{{ self::FOO }}
```

It will generate:

```rust
writer
    .write_fmt(
        format_args!(
            "{0}{1}{2}",
            &::askama::MarkupDisplay::new_unsafe(&(crate::FOO), ::askama::Html),
            &::askama::MarkupDisplay::new_unsafe(&(super::FOO), ::askama::Html),
            &::askama::MarkupDisplay::new_unsafe(&(self::FOO), ::askama::Html),
        ),
    )?;
::askama::Result::Ok(())
```

(Note: `crate::` is to get an item at the root level of the crate, `super::` is
to get an item in the parent module and `self::` is to get an item in the
current module.)

You can also access items from the type that implements `Template` as well using
as `Self::`, it'll use the same logic.

## Control blocks

### if/else

The generated code can be more complex than expected, as seen with `if`/`else`
conditions:

```jinja
{% if x == "a" %}
gateau
{% else %}
tarte
{% endif %}
```

It will generate:

```rust
if *(&(self.x == "a") as &bool) {
    writer.write_str("gateau")?;
} else {
    writer.write_str("tarte")?;
}
::askama::Result::Ok(())
```

Very much as expected except for the `&(self.x == "a") as &bool`. Now about why
the `as &bool` is needed:

The following syntax `*(&(...) as &bool)` is used to  trigger Rust's automatic
dereferencing, to coerce e.g. `&&&&&bool` to `bool`. First `&(...) as &bool`
coerces e.g. `&&&bool` to `&bool`. Then `*(&bool)` finally dereferences it to
`bool`.

In short, it allows to fallback to a boolean as much as possible, but it also
explains why you can't do:

```jinja
{% set x = "a" %}
{% if x %}
    {{ x }}
{% endif %}
```

Because it fail to compile because:

```console
error[E0605]: non-primitive cast: `&&str` as `&bool`
```

### if let

```jinja
{% if let Some(x) = x %}
    {{ x }}
{% endif %}
```

will generate:

```rust
if let Some(x) = &(self.x) {
    writer
        .write_fmt(
            format_args!(
                "{0}",
                &::askama::MarkupDisplay::new_unsafe(&(x), ::askama::Html),
            ),
        )?;
}
```

### Loops

```html
{% for user in users %}
    {{ user }}
{% endfor %}
```

will generate:

```rust
{
    let _iter = (&self.users).into_iter();
    for (user, _loop_item) in ::askama::helpers::TemplateLoop::new(_iter) {
        writer
            .write_fmt(
                format_args!(
                    "\n    {0}\n",
                    &::askama::MarkupDisplay::new_unsafe(&(user), ::askama::Html),
                ),
            )?;
    }
}
::askama::Result::Ok(())
```

Now let's see what happens if you add an `else` condition:

```jinja
{% for user in x if x.len() > 2 %}
    {{ user }}
{% else %}
    {{ x }}
{% endfor %}
```

Which generates:

```rust
{
    let mut _did_loop = false;
    let _iter = (&self.users).into_iter();
    for (user, _loop_item) in ::askama::helpers::TemplateLoop::new(_iter) {
        _did_loop = true;
        writer
            .write_fmt(
                format_args!(
                    "\n    {0}\n",
                    &::askama::MarkupDisplay::new_unsafe(&(user), ::askama::Html),
                ),
            )?;
    }
    if !_did_loop {
        writer
            .write_fmt(
                format_args!(
                    "\n    {0}\n",
                    &::askama::MarkupDisplay::new_unsafe(
                        &(self.x),
                        ::askama::Html,
                    ),
                ),
            )?;
    }
}
::askama::Result::Ok(())
```

It creates a `_did_loop` variable which will check if we entered the loop. If
we didn't (because the iterator didn't return any value), it will enter the
`else` condition by checking `if !_did_loop {`.

We can extend it even further if we add an `if` condition on our loop:

```jinja
{% for user in users if users.len() > 2 %}
    {{ user }}
{% else %}
    {{ x }}
{% endfor %}
```

which generates:

```rust
{
    let mut _did_loop = false;
    let _iter = (&self.users).into_iter();
    let _iter = _iter.filter(|user| -> bool { self.users.len() > 2 });
    for (user, _loop_item) in ::askama::helpers::TemplateLoop::new(_iter) {
        _did_loop = true;
        writer
            .write_fmt(
                format_args!(
                    "\n    {0}\n",
                    &::askama::MarkupDisplay::new_unsafe(&(user), ::askama::Html),
                ),
            )?;
    }
    if !_did_loop {
        writer
            .write_fmt(
                format_args!(
                    "\n    {0}\n",
                    &::askama::MarkupDisplay::new_unsafe(
                        &(self.x),
                        ::askama::Html,
                    ),
                ),
            )?;
    }
}
::askama::Result::Ok(())
```

It generates an iterator but filters it based on the `if` condition (`users.len() > 2`).
So once again, if the iterator doesn't return any value, we enter the `else`
condition.

Of course, if you only have a `if` and no `else`, the generated code is much
shorter:

```jinja
{% for user in users if users.len() > 2 %}
    {{ user }}
{% endfor %}
```

Which generates:

```rust
{
    let _iter = (&self.users).into_iter();
    let _iter = _iter.filter(|user| -> bool { self.users.len() > 2 });
    for (user, _loop_item) in ::askama::helpers::TemplateLoop::new(_iter) {
        writer
            .write_fmt(
                format_args!(
                    "\n    {0}\n",
                    &::askama::MarkupDisplay::new_unsafe(&(user), ::askama::Html),
                ),
            )?;
    }
}
::askama::Result::Ok(())
```

## Filters

Example of using the `abs` built-in filter:

```jinja
{{ -2|abs }}
```

Which generates:

```rust
writer
    .write_fmt(
        format_args!(
            "{0}",
            &::askama::MarkupDisplay::new_unsafe(
                &(::askama::filters::abs(-2)?),
                ::askama::Html,
            ),
        ),
    )?;
::askama::Result::Ok(())
```

The filter is called with `-2` as first argument. You can add further arguments
to the call like this:

```jinja
{{ "a"|indent(4) }}
```

Which generates:

```rust
writer
    .write_fmt(
        format_args!(
            "{0}",
            &::askama::MarkupDisplay::new_unsafe(
                &(::askama::filters::indent("a", 4)?),
                ::askama::Html,
            ),
        ),
    )?;
::askama::Result::Ok(())
```

No surprise there, `4` is added after `"a"`. Now let's check when we chain the filters:

```jinja
{{ "a"|indent(4)|capitalize }}
```

Which generates:

```rust
writer
    .write_fmt(
        format_args!(
            "{0}",
            &::askama::MarkupDisplay::new_unsafe(
                &(::askama::filters::capitalize(
                    &(::askama::filters::indent("a", 4)?),
                )?),
                ::askama::Html,
            ),
        ),
    )?;
::askama::Result::Ok(())
```

As expected, `capitalize`'s first argument is the value returned by the `indent` call.

## Macros

This code:

```html
{% macro heading(arg) %}
<h1>{{arg}}</h1>
{% endmacro %}

{% call heading("title") %}
```

generates:

```rust
{
    let (arg) = (("title"));
    writer
        .write_fmt(
            format_args!(
                "\n<h1>{0}</h1>\n",
                &::askama::MarkupDisplay::new_unsafe(&(arg), ::askama::Html),
            ),
        )?;
}
::askama::Result::Ok(())
```

As you can see, the macro itself isn't present in the generated code, only its
internal code is generated as well as its arguments.
