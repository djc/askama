use path;

use std::borrow::Cow;
use std::path::PathBuf;

use syn;


pub struct TemplateInput<'a> {
    pub ast: &'a syn::DeriveInput,
    pub meta: TemplateMeta<'a>,
    pub path: PathBuf,
    pub source: Cow<'a, str>,
}

impl<'a> TemplateInput<'a> {
    pub fn new(ast: &'a syn::DeriveInput) -> TemplateInput<'a> {
        let meta = TemplateMeta::new(ast);
        let (path, source) = match meta.source {
            Source::Source(s) => (PathBuf::new(), Cow::Borrowed(s)),
            Source::Path(s) => {
                let path = path::find_template_from_path(&s, None);
                let src = path::get_template_source(&path);
                (path, Cow::Owned(src))
            },
        };
        TemplateInput { ast, meta, path, source }
    }
}

// Holds metadata for the template, based on the `template()` attribute.
pub struct TemplateMeta<'a> {
    source: Source<'a>,
    pub print: Print,
    pub escaping: EscapeMode,
}

impl<'a> TemplateMeta<'a> {
    fn new(ast: &'a syn::DeriveInput) -> TemplateMeta<'a> {
        let attr = ast.attrs.iter().find(|a| a.name() == "template");
        if attr.is_none() {
            let msg = format!("'template' attribute not found on struct '{}'",
                              ast.ident.as_ref());
            panic!(msg);
        }

        let attr = attr.unwrap();
        let mut source = None;
        let mut print = Print::None;
        let mut escaping = EscapeMode::Html;
        if let syn::MetaItem::List(_, ref inner) = attr.value {
            for nm_item in inner {
                if let syn::NestedMetaItem::MetaItem(ref item) = *nm_item {
                    if let syn::MetaItem::NameValue(ref key, ref val) = *item {
                        match key.as_ref() {
                            "path" => if let syn::Lit::Str(ref s, _) = *val {
                                source = Some(Source::Path(s.as_ref()));
                            } else {
                                panic!("template path must be string literal");
                            },
                            "source" => if let syn::Lit::Str(ref s, _) = *val {
                                source = Some(Source::Source(s.as_ref()));
                            } else {
                                panic!("template source must be string literal");
                            },
                            "print" => if let syn::Lit::Str(ref s, _) = *val {
                                print = s.into();
                            } else {
                                panic!("print value must be string literal");
                            },
                            "escape" => if let syn::Lit::Str(ref s, _) = *val {
                                escaping = s.into();
                            } else {
                                panic!("escape value must be string literal");
                            },
                            _ => { panic!("unsupported annotation key found") }
                        }
                    }
                }
            }
        }

        match source {
            Some(s) => TemplateMeta { source: s, print, escaping },
            None => panic!("template path or source not found in struct attributes"),
        }
    }
}

enum Source<'a> {
    Path(&'a str),
    Source(&'a str),
}

#[derive(PartialEq)]
pub enum EscapeMode {
    Html,
    None,
}

impl<'a> From<&'a String> for EscapeMode {
    fn from(s: &'a String) -> EscapeMode {
        use self::EscapeMode::*;
        match s.as_ref() {
            "html" => Html,
            "none" => None,
            v => panic!("invalid value for escape option: {}", v),
        }
    }
}

#[derive(PartialEq)]
pub enum Print {
    All,
    Ast,
    Code,
    None,
}

impl<'a> From<&'a String> for Print {
    fn from(s: &'a String) -> Print {
        use self::Print::*;
        match s.as_ref() {
            "all" => All,
            "ast" => Ast,
            "code" => Code,
            "none" => None,
            v => panic!("invalid value for print option: {}", v),
        }
    }
}
