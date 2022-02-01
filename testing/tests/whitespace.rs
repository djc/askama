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
<<<<<<< HEAD
    assert_eq!(template.render().unwrap(), "\n0\n0\n0\n0\n\n\n\n0\n0\n0\n0\n0\n\na0\na1\nvalue\n\n\n\n\n\n[\n  \"a0\",\n  \"a1\",\n  \"a2\",\n  \"a3\"\n]\n[\n  \"a0\",\n  \"a1\",\n  \"a2\",\n  \"a3\"\n][\n  \"a0\",\n  \"a1\",\n  \"a2\",\n  \"a3\"\n]\n[\n  \"a1\"\n][\n  \"a1\"\n]\n[\n  \"a1\",\n  \"a2\"\n][\n  \"a1\",\n  \"a2\"\n]\n[\n  \"a1\"\n][\n  \"a1\"\n]1-1-1\n3333 3\n2222 2\n0000 0\n3333 3\n\ntruefalse\nfalsefalsefalse\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n");
=======
    assert_eq!(template.render().unwrap(), "\n0\n0\n0\n0\n\n\n\n0\n0\n0\n0\n0\n\na0\na1\nvalue\n\n\n\n\n\n[\n  &quot;a0&quot;,\n  &quot;a1&quot;,\n  &quot;a2&quot;,\n  &quot;a3&quot;\n]\n[\n  &quot;a0&quot;,\n  &quot;a1&quot;,\n  &quot;a2&quot;,\n  &quot;a3&quot;\n][\n  &quot;a0&quot;,\n  &quot;a1&quot;,\n  &quot;a2&quot;,\n  &quot;a3&quot;\n]\n[\n  &quot;a1&quot;\n][\n  &quot;a1&quot;\n]\n[\n  &quot;a1&quot;,\n  &quot;a2&quot;\n][\n  &quot;a1&quot;,\n  &quot;a2&quot;\n]\n[\n  &quot;a1&quot;\n][\n  &quot;a1&quot;\n]1-1-1\n3333 3\n2222 2\n0000 0\n3333 3\n\ntruefalse\nfalsefalsefalse\n\n\n\n\n\n\n\n\n\n\n\n\n\n");
>>>>>>> 29f0c06 (Make json filter safe)
}
