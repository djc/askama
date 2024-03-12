use askama::Template;

macro_rules! test_size {
    ($source:literal, $expected:expr) => {{
        #[derive(Template)]
        #[allow(dead_code)]
        #[template(source = $source, ext = "txt")]
        struct T(bool);

        assert_eq!(T::SIZE_HINT, $expected);
    }};
}

#[test]
fn test_cond_size_hint() {
    test_size!("{% if self.0 %}12345{% else %}12345{% endif %}", 10);
}

#[test]
fn test_match_size_hint() {
    test_size!(
        "{% match self.0 %}{% when true %}12345{% else %}12345{% endmatch %}",
        5
    );
}

#[test]
fn test_loop_size_hint() {
    test_size!("{% for i in 0..1 %}12345{% endfor %}", 7);
}

#[test]
fn test_block_size_hint() {
    #[derive(Template)]
    #[template(path = "size-child.txt")]
    struct T;

    assert_eq!(T::SIZE_HINT, 3);
}

#[test]
fn test_super_size_hint() {
    #[derive(Template)]
    #[template(path = "size-child-super.txt")]
    struct T;

    assert_eq!(T::SIZE_HINT, 5);
}
