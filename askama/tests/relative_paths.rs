#[cfg(feature = "relative-paths")]
mod relative_paths {
    use askama::Template;

    #[derive(Template)]
    #[template(path = "relative_paths.txt")]
    struct RelativePathTemplate {
        name: String,
    }

    #[test]
    fn test_relative_paths() {
        let t = RelativePathTemplate {
            name: "world".to_string(),
        };
        assert_eq!(t.render().unwrap(), "Hello, world!");
    }
}
