#[macro_export]
macro_rules! map {
    ($self:ident, $name:ident, $prop:ident) => {{
        let mut v = Vec::new();
        for i in &$self.$name {
            if let Some(i) = i.$prop {
                v.push(i);
            }
        }
        v
    }};

    ($self:ident, $name:ident, $prop:ident, $callback:expr) => {{
        let mut v = Vec::new();
        let c = $callback;
        for i in &$self.$name {
            if let Some(i) = i.$prop {
                v.push(c(i));
            }
        }
        v
    }};
}

#[cfg(test)]
mod test {
    #[test]
    fn test_map() {
        struct Test<'a> {
            a: Vec<Bar<'a>>,
        }

        struct Bar<'a> {
            b: Option<&'a str>,
        }

        let a = &Test {
            a: vec![
                Bar { b: Some("foo") },
                Bar { b: None },
                Bar { b: Some("bar") },
            ],
        };

        assert_eq!(map!(a, a, b), vec!["foo", "bar"]);
        assert_eq!(
            map!(a, a, b, |s: &str| s.to_uppercase()),
            vec!["FOO", "BAR"]
        );
    }
}
