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
            Source::Source(s) => {
                let path = match meta.ext {
                    Some(v) => PathBuf::from(format!("_.{}", v)),
                    None => PathBuf::new(),
                };
                (path, Cow::Borrowed(s))
            },
            Source::Path(s) => {
                let path = path::find_template_from_path(s, None);
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
    pub ext: Option<&'a str>,
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
        let mut ext = None;
        if let syn::MetaItem::List(_, ref inner) = attr.value {
            for nm_item in inner {
                if let syn::NestedMetaItem::MetaItem(ref item) = *nm_item {
                    if let syn::MetaItem::NameValue(ref key, ref val) = *item {
                        match key.as_ref() {
                            "path" => if let syn::Lit::Str(ref s, _) = *val {
                                if source.is_some() {
                                    panic!("must specify 'source' or 'path', not both");
                                }
                                source = Some(Source::Path(s.as_ref()));
                            } else {
                                panic!("template path must be string literal");
                            },
                            "source" => if let syn::Lit::Str(ref s, _) = *val {
                                if source.is_some() {
                                    panic!("must specify 'source' or 'path', not both");
                                }
                                source = Some(Source::Source(s.as_ref()));
                            } else {
                                panic!("template source must be string literal");
                            },
                            "print" => if let syn::Lit::Str(ref s, _) = *val {
                                print = (s.as_ref() as &str).into();
                            } else {
                                panic!("print value must be string literal");
                            },
                            "escape" => if let syn::Lit::Str(ref s, _) = *val {
                                escaping = (s.as_ref() as &str).into();
                            } else {
                                panic!("escape value must be string literal");
                            },
                            "ext" => if let syn::Lit::Str(ref s, _) = *val {
                                ext = Some((s.as_ref() as &str).into());
                            } else {
                                panic!("ext value must be string literal");
                            },
                            _ => { panic!("unsupported annotation key found") }
                        }
                    }
                }
            }
        }

        match source {
            Some(s) => {
                if let Source::Path(_) = s {
                    if ext.is_some() {
                        panic!("'ext' attribute cannot be used with 'path' attribute");
                    }
                }
                TemplateMeta { source: s, print, escaping, ext }
            },
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

impl<'a> From<&'a str> for EscapeMode {
    fn from(s: &'a str) -> EscapeMode {
        use self::EscapeMode::*;
        match s {
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

impl<'a> From<&'a str> for Print {
    fn from(s: &'a str) -> Print {
        use self::Print::*;
        match s {
            "all" => All,
            "ast" => Ast,
            "code" => Code,
            "none" => None,
            v => panic!("invalid value for print option: {}", v),
        }
    }
}
