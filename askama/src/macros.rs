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

    ($self:ident, $name:ident, $prop:ident, $callback:path) => {{
        let mut v = Vec::new();
        for i in &$self.$name {
            if let Some(i) = i.$prop {
                v.push($callback(i).unwrap());
            }
        }
        v
    }};

    ($self:ident, $name:ident, $prop:ident, & $callback:path) => {{
        let mut v = Vec::new();
        for i in &$self.$name {
            if let Some(i) = i.$prop {
                v.push($callback(&i).unwrap());
            }
        }
        v
    }};
}

#[cfg(test)]
mod test {
    use super::super::{filters, Error};
    #[test]
    fn test_map() {
        struct Test<'a> {
            a: Vec<Bar<'a>>,
        }

        struct Bar<'a> {
            b: Option<&'a str>,
            n: Option<i32>,
        }

        let a = &Test {
            a: vec![
                Bar {
                    b: Some("foo"),
                    n: Some(1),
                },
                Bar {
                    b: None,
                    n: Some(-1),
                },
                Bar {
                    b: Some("bar"),
                    n: None,
                },
            ],
        };

        assert_eq!(map!(a, a, b), vec!["foo", "bar"]);
        assert_eq!(map!(a, a, b, &filters::uppercase), vec!["FOO", "BAR"]);

        let c = map!(a, a, n, filters::abs);
        assert_eq!(c[0], 1 as i32);
        assert_eq!(c[1], 1 as i32);
    }
}
