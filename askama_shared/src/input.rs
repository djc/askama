use path;

use std::borrow::Cow;
use std::path::{Path, PathBuf};

use syn;

use parser::{self, Node};


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
        let mut escaping = None;
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
                                escaping = Some((s.as_ref() as &str).into());
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

        let source = source.expect("template path or source not found in attributes");
        match (&source, ext.is_some()) {
            (&Source::Path(_), true) => {
                panic!("'ext' attribute cannot be used with 'path' attribute")
            },
            (&Source::Source(_), false) => {
                panic!("must include 'ext' attribute when using 'source' attribute")
            },
            _ => {},
        }
        let escaping = match escaping {
            Some(m) => m,
            None => {
                let ext = match source {
                    Source::Path(p) => {
                        Path::new(p).extension().map(|s| s.to_str().unwrap()).unwrap_or("")
                    },
                    Source::Source(_) => ext.unwrap(), // Already panicked if None
                };
                if HTML_EXTENSIONS.contains(&ext) {
                    EscapeMode::Html
                } else {
                    EscapeMode::None
                }
            }
        };
        TemplateMeta { source, print, escaping, ext }
    }
}

pub struct Imports<'a> {
    pub sources: Vec<Cow<'a, str>>
}

impl <'a> Imports<'a> {
    pub fn new(parent_nodes: &'a [Node], parent_path: &'a Path) -> Imports<'a> {
        let sources = parent_nodes.iter().filter_map(|n| {
            match *n {
                Node::Import(_, ref import_path) => {
                    let path = path::find_template_from_path(import_path, Some(parent_path));
                    let src = path::get_template_source(&path);
                    Some(Cow::Owned(src))
                },
                _ => None,
            }
        }).collect();
        Imports {
            sources,
        }
    }

    pub fn parse(&'a self) -> Vec<Node<'a>> {
        self.sources.iter()
            .flat_map(|s| parser::parse(s.as_ref()))
            .collect()
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

const HTML_EXTENSIONS: [&str; 3] = ["html", "htm", "xml"];
