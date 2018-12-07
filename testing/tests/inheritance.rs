extern crate askama;

use askama::Template;

#[derive(Template)]
#[template(path = "base.html")]
struct BaseTemplate<'a> {
    title: &'a str,
}

#[derive(Template)]
#[template(path = "child.html")]
struct ChildTemplate<'a> {
    _parent: BaseTemplate<'a>,
}

#[test]
fn test_use_base_directly() {
    let t = BaseTemplate { title: "Foo" };
    assert_eq!(t.render().unwrap(), "Foo\n\nFoo\nCopyright 2017");
}

#[test]
fn test_simple_extends() {
    let t = ChildTemplate {
        _parent: BaseTemplate { title: "Bar" },
    };
    assert_eq!(
        t.render().unwrap(),
        "Bar\n(Bar) Content goes here\nFoo\nCopyright 2017"
    );
}

pub mod parent {
    use askama::Template;
    #[derive(Template)]
    #[template(path = "base.html")]
    pub struct BaseTemplate<'a> {
        pub title: &'a str,
    }
}

pub mod child {
    use super::parent::*;
    use askama::Template;
    #[derive(Template)]
    #[template(path = "child.html")]
    pub struct ChildTemplate<'a> {
        pub _parent: BaseTemplate<'a>,
    }
}

#[test]
fn test_different_module() {
    let t = child::ChildTemplate {
        _parent: parent::BaseTemplate { title: "a" },
    };
    assert_eq!(
        t.render().unwrap(),
        "a\n(a) Content goes here\nFoo\nCopyright 2017"
    );
}

#[derive(Template)]
#[template(path = "nested-base.html")]
struct NestedBaseTemplate {}

#[derive(Template)]
#[template(path = "nested-child.html")]
struct NestedChildTemplate {
    _parent: NestedBaseTemplate,
}

#[test]
fn test_nested_blocks() {
    let t = NestedChildTemplate {
        _parent: NestedBaseTemplate {},
    };
    assert_eq!(t.render().unwrap(), "\ndurpy\n");
}

#[derive(Template)]
#[template(path = "deep-base.html")]
struct DeepBaseTemplate {
    year: u16,
}

#[derive(Template)]
#[template(path = "deep-mid.html")]
struct DeepMidTemplate {
    _parent: DeepBaseTemplate,
    title: String,
}

#[derive(Template)]
#[template(path = "deep-kid.html")]
struct DeepKidTemplate {
    _parent: DeepMidTemplate,
    item: String,
}

#[test]
fn test_deep() {
    let t = DeepKidTemplate {
        _parent: DeepMidTemplate {
            _parent: DeepBaseTemplate { year: 2018 },
            title: "Test".into(),
        },
        item: "Foo".into(),
    };

    assert_eq!(
        t.render().unwrap(),
        "
<html>
  <head>
  
  <script></script>

  </head>
  <body>
  
  <div id=\"wrap\">
    <section id=\"content\">
    
  Foo Foo Foo

    </section>
    <section id=\"nav\">
      nav nav nav
    </section>
  </div>

  </body>
</html>"
    );
    assert_eq!(
        t._parent.render().unwrap(),
        "
<html>
  <head>
  
  Test
  
    <style></style>
  

  </head>
  <body>
  
  <div id=\"wrap\">
    <section id=\"content\">
    
      No content found
    
    </section>
    <section id=\"nav\">
      nav nav nav
    </section>
  </div>

  </body>
</html>"
    );
    assert_eq!(
        t._parent._parent.render().unwrap(),
        "
<html>
  <head>
  
    <style></style>
  
  </head>
  <body>
  
    nav nav nav
    Copyright 2018
  
  </body>
</html>"
    );
}

#[derive(Template)]
#[template(path = "deep-base.html")]
struct FlatDeepBaseTemplate {
    year: u16,
}

#[derive(Template)]
#[template(path = "deep-mid.html")]
struct FlatDeepMidTemplate {
    title: String,
}

#[derive(Template)]
#[template(path = "deep-kid.html")]
struct FlatDeepKidTemplate {
    item: String,
}

#[test]
fn test_flat_deep() {
    let t = FlatDeepKidTemplate { item: "Foo".into() };

    assert_eq!(
        t.render().unwrap(),
        "
<html>
  <head>
  
  <script></script>

  </head>
  <body>
  
  <div id=\"wrap\">
    <section id=\"content\">
    
  Foo Foo Foo

    </section>
    <section id=\"nav\">
      nav nav nav
    </section>
  </div>

  </body>
</html>"
    );

    let t = FlatDeepMidTemplate {
        title: "Test".into(),
    };
    assert_eq!(
        t.render().unwrap(),
        "
<html>
  <head>
  
  Test
  
    <style></style>
  

  </head>
  <body>
  
  <div id=\"wrap\">
    <section id=\"content\">
    
      No content found
    
    </section>
    <section id=\"nav\">
      nav nav nav
    </section>
  </div>

  </body>
</html>"
    );

    let t = FlatDeepBaseTemplate { year: 2018 };
    assert_eq!(
        t.render().unwrap(),
        "
<html>
  <head>
  
    <style></style>
  
  </head>
  <body>
  
    nav nav nav
    Copyright 2018
  
  </body>
</html>"
    );
}
