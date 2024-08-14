// Files containing tests for generated code.

use crate::build_template;
use std::fmt::Write;

#[test]
fn check_if_let() {
    // This function makes it much easier to compare expected code by adding the wrapping around
    // the code we want to check.
    fn compare(jinja: &str, expected: &str) {
        let jinja = format!(
            r##"#[template(source = r#"{jinja}"#, ext = "txt")]
struct Foo;"##
        );
        let generated =
            build_template(&syn::parse_str::<syn::DeriveInput>(&jinja).unwrap()).unwrap();

        let generated_s = syn::parse_str::<proc_macro2::TokenStream>(&generated)
            .unwrap()
            .to_string();
        let mut new_expected = String::with_capacity(expected.len());
        for line in expected.split('\n') {
            new_expected.write_fmt(format_args!("{line}\n")).unwrap();
        }
        let expected = format!(
            r#"impl ::askama::Template for Foo {{
    fn render_into(&self, writer: &mut (impl ::std::fmt::Write + ?Sized)) -> ::askama::Result<()> {{
        {new_expected}
        ::askama::Result::Ok(())
    }}
    const EXTENSION: ::std::option::Option<&'static ::std::primitive::str> = Some("txt");
    const SIZE_HINT: ::std::primitive::usize = 3;
    const MIME_TYPE: &'static ::std::primitive::str = "text/plain; charset=utf-8";
}}
impl ::std::fmt::Display for Foo {{
    #[inline]
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {{
        ::askama::Template::render_into(self, f).map_err(|_| ::std::fmt::Error {{}})
    }}
}}"#
        );
        let expected_s = syn::parse_str::<proc_macro2::TokenStream>(&expected)
            .unwrap()
            .to_string();
        assert_eq!(
            generated_s, expected_s,
            "=== Expected ===\n{}\n=== Found ===\n{}\n=====",
            generated, expected
        );
    }

    // In this test, we ensure that `query` never is `self.query`.
    compare(
        "{% if let Some(query) = s && !query.is_empty() %}{{query}}{% endif %}",
        r#"if let Some(query,) = &self.s && !query.is_empty() {
    ::std::write!(
        writer,
        "{expr0}",
        expr0 = &::askama::MarkupDisplay::new_unsafe(&(query), ::askama::Text),
    )?;
}"#,
    );

    // In this test, we ensure that `s` is `self.s` only in the first `if let Some(s) = self.s`
    // condition.
    compare(
        "{% if let Some(s) = s %}{{ s }}{% endif %}",
        r#"if let Some(s,) = &self.s {
    ::std::write!(
        writer,
        "{expr0}",
        expr0 = &::askama::MarkupDisplay::new_unsafe(&(s), ::askama::Text),
    )?;
}"#,
    );

    // In this test, we ensure that `s` is `self.s` only in the first `if let Some(s) = self.s`
    // condition.
    compare(
        "{% if let Some(s) = s && !s.is_empty() %}{{s}}{% endif %}",
        r#"if let Some(s,) = &self.s && !s.is_empty() {
    ::std::write!(
        writer,
        "{expr0}",
        expr0 = &::askama::MarkupDisplay::new_unsafe(&(s), ::askama::Text),
    )?;
}"#,
    );
}
