#![cfg(feature = "serde-json")]

use askama::Template;

#[derive(askama::Template, Default)]
#[template(path = "allow-whitespaces.html")]
struct AllowWhitespaces {
    tuple: (u64, u64, u64, u64),
    string: &'static str,
    option: Option<bool>,
    nested_1: AllowWhitespacesNested1,
}

#[derive(Default)]
struct AllowWhitespacesNested1 {
    nested_2: AllowWhitespacesNested2,
}

#[derive(Default)]
struct AllowWhitespacesNested2 {
    array: &'static [&'static str],
    hash: std::collections::HashMap<&'static str, &'static str>,
}

impl AllowWhitespaces {
    fn f0(&self) -> &str {
        ""
    }
    fn f1(&self, _a: &str) -> &str {
        ""
    }
    fn f2(&self, _a: &str, _b: &str) -> &str {
        ""
    }
}

#[test]
fn test_extra_whitespace() {
    let mut template = AllowWhitespaces::default();
    template.nested_1.nested_2.array = &["a0", "a1", "a2", "a3"];
    template.nested_1.nested_2.hash.insert("key", "value");
    assert_eq!(template.render().unwrap(), "\n0\n0\n0\n0\n\n\n\n0\n0\n0\n0\n0\n\na0\na1\nvalue\n\n\n\n\n\n[\n  &quot;a0&quot;,\n  &quot;a1&quot;,\n  &quot;a2&quot;,\n  &quot;a3&quot;\n]\n[\n  &quot;a0&quot;,\n  &quot;a1&quot;,\n  &quot;a2&quot;,\n  &quot;a3&quot;\n][\n  &quot;a0&quot;,\n  &quot;a1&quot;,\n  &quot;a2&quot;,\n  &quot;a3&quot;\n]\n[\n  &quot;a1&quot;\n][\n  &quot;a1&quot;\n]\n[\n  &quot;a1&quot;,\n  &quot;a2&quot;\n][\n  &quot;a1&quot;,\n  &quot;a2&quot;\n]\n[\n  &quot;a1&quot;\n][\n  &quot;a1&quot;\n]1-1-1\n3333 3\n2222 2\n0000 0\n3333 3\n\ntruefalse\nfalsefalsefalse\n\n\n\n\n\n\n\n\n\n\n\n\n\n");
}

macro_rules! test_template_minimize {
    ($source:literal, $rendered:expr) => {{
        #[derive(Template)]
        #[template(source = $source, ext = "txt", config = "test_minimize.toml")]
        struct CondWs;

        assert_eq!(CondWs.render().unwrap(), $rendered);
    }};
}

macro_rules! test_template {
    ($source:literal, $rendered:expr) => {{
        #[derive(Template)]
        #[template(source = $source, ext = "txt")]
        struct CondWs;

        assert_eq!(CondWs.render().unwrap(), $rendered);
    }};
}

#[test]
fn test_minimize_whitespace() {
    test_template_minimize!(
        "\n1\r\n{%  if true  %}\n\n2\r\n\r\n{%  endif  %} 3\r\n\r\n\r\n",
        "\n1\n\n2\n 3\r\n\r\n\r\n"
    );
    test_template_minimize!(
        "\n1\r\n{%+  if true  %}\n\n2\r\n\r\n{%  endif  %} 3\r\n\r\n\r\n",
        "\n1\r\n\n2\n 3\r\n\r\n\r\n"
    );
    test_template_minimize!(
        "\n1\r\n{%-  if true  %}\n\n2\r\n\r\n{%  endif  %} 3\r\n\r\n\r\n",
        "\n1\n2\n 3\r\n\r\n\r\n"
    );
    test_template_minimize!(" \n1 \n{%  if true  %} 2 {%  endif  %}3 ", " \n1\n 2 3 ");

    test_template!(
        "\n1\r\n{%~  if true  ~%}\n\n2\r\n\r\n{%~  endif  ~%} 3\r\n\r\n\r\n",
        "\n1\n\n2\n 3\r\n\r\n\r\n"
    );
    test_template!(
        " \n1 \n{%~  if true  ~%} 2 {%~  endif  ~%}3 ",
        " \n1\n 2 3 "
    );
}

macro_rules! test_template_config {
    ($config:literal, $source:literal, $rendered: literal) => {{
        #[derive(Template)]
        #[template(source = $source, ext = "txt", config = $config)]
        struct CondWs;

        assert_eq!(CondWs.render().unwrap(), $rendered);
    }};
}

#[test]
fn test_outer_whitespace() {
    test_template_config!("test_trim.toml", "\t1\t\t", "\t1\t\t");
    test_template_config!("test_trim.toml", " 1 ", " 1 ");
    test_template_config!("test_trim.toml", "\n1\n\n\n", "\n1\n\n\n");
    test_template_config!("test_trim.toml", "\t1{# #}\t", "\t1");
    test_template_config!("test_trim.toml", " 1{# #} ", " 1");
    test_template_config!("test_trim.toml", "\n1{# #}\n\n\n", "\n1");
    test_template_minimize!("\t1{# #} ", "\t1 ");
    test_template_minimize!("\t1{# #}\t", "\t1 ");
    test_template_minimize!("\t1{# #}  ", "\t1 ");
    test_template_minimize!("\t1{# #}\t\t", "\t1 ");
    test_template_minimize!(" 1{# #} ", " 1 ");
    test_template_minimize!("\n1{# #}\n\n\n", "\n1\n");
    test_template!("\t1{# #}\t", "\t1\t");
    test_template!(" 1{# #} ", " 1 ");
    test_template!("\n1{# #}\n\n\n", "\n1\n\n\n");
}

macro_rules! test_template_ws_config {
    ($config:literal, $ws:literal, $source:literal, $rendered: literal) => {{
        #[derive(Template)]
        #[template(source = $source, ext = "txt", config = $config, whitespace = $ws)]
        struct CondWs;

        assert_eq!(CondWs.render().unwrap(), $rendered);
    }};
}

#[test]
fn test_template_whitespace_config() {
    test_template_ws_config!("test_trim.toml", "preserve", "\t1{# #}\t2", "\t1\t2");
    test_template_ws_config!("test_trim.toml", "minimize", " 1{# #}  2", " 1 2");
    test_template_ws_config!("test_trim.toml", "suppress", " 1{# #}  2", " 12");
    test_template_ws_config!(
        "test_minimize.toml",
        "preserve",
        "\n1{# #}\n\n\n2",
        "\n1\n\n\n2"
    );
    test_template_ws_config!("test_minimize.toml", "suppress", "\n1{# #}\n\n\n2", "\n12");
}
