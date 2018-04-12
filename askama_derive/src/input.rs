use shared::path;

use std::path::{Path, PathBuf};

use syn;


pub struct TemplateInput<'a> {
    pub ast: &'a syn::DeriveInput,
    pub meta: TemplateMeta,
    pub path: PathBuf,
    pub source: String,
}

impl<'a> TemplateInput<'a> {
    pub fn new(ast: &'a syn::DeriveInput) -> TemplateInput<'a> {
        let meta = TemplateMeta::new(ast);
        let (path, source) = match meta.source {
            Source::Source(ref s) => {
                let path = match meta.ext {
                    Some(ref v) => PathBuf::from(format!("_.{}", v)),
                    None => PathBuf::new(),
                };
                (path, s.clone())
            },
            Source::Path(ref s) => {
                let path = path::find_template_from_path(s, None);
                let src = path::get_template_source(&path);
                (path, src)
            },
        };
        TemplateInput { ast, meta, path, source }
    }
}

// Holds metadata for the template, based on the `template()` attribute.
pub struct TemplateMeta {
    source: Source,
    pub print: Print,
    pub escaping: EscapeMode,
    pub ext: Option<String>,
}

impl TemplateMeta {
    fn new(ast: &syn::DeriveInput) -> TemplateMeta {
        let attr = ast.attrs
            .iter()
            .find(|a| a.interpret_meta().unwrap().name() == "template");
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
        if let syn::Meta::List(ref inner) = attr.interpret_meta().unwrap() {
            for nm_item in inner.nested.iter() {
                if let syn::NestedMeta::Meta(ref item) = *nm_item {
                    if let syn::Meta::NameValue(ref pair) = *item {
                        match pair.ident.as_ref() {
                            "path" => if let syn::Lit::Str(ref s) = pair.lit {
                                if source.is_some() {
                                    panic!("must specify 'source' or 'path', not both");
                                }
                                source = Some(Source::Path(s.value()));
                            } else {
                                panic!("template path must be string literal");
                            },
                            "source" => if let syn::Lit::Str(ref s) = pair.lit {
                                if source.is_some() {
                                    panic!("must specify 'source' or 'path', not both");
                                }
                                source = Some(Source::Source(s.value()));
                            } else {
                                panic!("template source must be string literal");
                            },
                            "print" => if let syn::Lit::Str(ref s) = pair.lit {
                                print = s.value().into();
                            } else {
                                panic!("print value must be string literal");
                            },
                            "escape" => if let syn::Lit::Str(ref s) = pair.lit {
                                escaping = Some(s.value().into());
                            } else {
                                panic!("escape value must be string literal");
                            },
                            "ext" => if let syn::Lit::Str(ref s) = pair.lit {
                                ext = Some(s.value());
                            } else {
                                panic!("ext value must be string literal");
                            },
                            attr => panic!("unsupported annotation key '{}' found", attr),
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
                    Source::Path(ref p) =>
                        Path::new(p).extension().map(|s| s.to_str().unwrap()).unwrap_or(""),
                    Source::Source(_) => ext.as_ref().unwrap(), // Already panicked if None
                };
                if HTML_EXTENSIONS.contains(&ext) {
                    EscapeMode::Html
                } else {
                    EscapeMode::None
                }
            },
        };
        TemplateMeta { source, print, escaping, ext }
    }
}

enum Source {
    Path(String),
    Source(String),
}

#[derive(PartialEq)]
pub enum EscapeMode {
    Html,
    None,
}

impl From<String> for EscapeMode {
    fn from(s: String) -> EscapeMode {
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

impl From<String> for Print {
    fn from(s: String) -> Print {
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

const HTML_EXTENSIONS: [&str; 3] = ["html", "htm", "xml"];
