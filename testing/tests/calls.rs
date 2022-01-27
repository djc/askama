use askama::Template;

#[derive(Template)]
#[template(source = "{{ func(value) }}", ext = "txt")]
struct OneFunction {
    func: fn(&i32) -> i32,
    value: i32,
}

#[test]
fn test_one_func() {
    let t = OneFunction {
        func: |&i| 2 * i,
        value: 123,
    };
    assert_eq!(t.render().unwrap(), "246");
}

#[derive(Template)]
#[template(source = "{{ self.func(value) }}", ext = "txt")]
struct OneFunctionSelf {
    value: i32,
}

impl OneFunctionSelf {
    fn func(&self, i: &i32) -> i32 {
        2 * i
    }
}

#[test]
fn test_one_func_self() {
    let t = OneFunctionSelf { value: 123 };
    assert_eq!(t.render().unwrap(), "246");
}

#[derive(Template)]
#[template(source = "{{ func[index](value) }}", ext = "txt")]
struct OneFunctionIndex<'a> {
    func: &'a [fn(&i32) -> i32],
    value: i32,
    index: usize,
}

#[test]
fn test_one_func_index() {
    let t = OneFunctionIndex {
        func: &[|_| panic!(), |&i| 2 * i, |_| panic!(), |_| panic!()],
        value: 123,
        index: 1,
    };
    assert_eq!(t.render().unwrap(), "246");
}

struct AddToGetAFunction;

impl std::ops::Add<usize> for &AddToGetAFunction {
    type Output = fn(&i32) -> i32;

    fn add(self, rhs: usize) -> Self::Output {
        assert_eq!(rhs, 1);
        |&i| 2 * i
    }
}

#[derive(Template)]
#[template(source = "{{ (func + index)(value) }}", ext = "txt")]
struct OneFunctionBinop<'a> {
    func: &'a AddToGetAFunction,
    value: i32,
    index: usize,
}

#[test]
fn test_one_func_binop() {
    let t = OneFunctionBinop {
        func: &AddToGetAFunction,
        value: 123,
        index: 1,
    };
    assert_eq!(t.render().unwrap(), "246");
}
