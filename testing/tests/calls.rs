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

fn double_attr_arg_helper(x: u32) -> u32 {
    x * x + x
}

#[derive(askama::Template)]
#[template(
    source = "{{ self::double_attr_arg_helper(self.x.0 + 2) }}",
    ext = "txt"
)]
struct DoubleAttrArg {
    x: (u32,),
}

#[test]
fn test_double_attr_arg() {
    let t = DoubleAttrArg { x: (10,) };
    assert_eq!(t.render().unwrap(), "156");
}
