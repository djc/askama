use proc_macro2::TokenStream;

use quote::ToTokens;

use shared::path;

use std::path::{Path, PathBuf};

use syn;

pub struct TemplateInput<'a> {
    pub ast: &'a syn::DeriveInput,
    pub source: Source,
    pub print: Print,
    pub escaping: EscapeMode,
    pub ext: Option<String>,
    pub parent: Option<&'a syn::Type>,
    pub path: PathBuf,
}

impl<'a> TemplateInput<'a> {
    pub fn new(ast: &'a syn::DeriveInput) -> TemplateInput<'a> {
        let mut meta = None;
        for attr in &ast.attrs {
            match attr.interpret_meta() {
                Some(m) => if m.name() == "template" {
                    meta = Some(m)
                },
                None => {
                    let mut tokens = TokenStream::new();
                    attr.to_tokens(&mut tokens);
                    panic!("unable to interpret attribute: {}", tokens)
                }
            }
        }

        let meta_list = match meta.expect("no attribute 'template' found") {
            syn::Meta::List(inner) => inner,
            _ => panic!("attribute 'template' has incorrect type"),
        };

        let mut source = None;
        let mut print = Print::None;
        let mut escaping = None;
        let mut ext = None;
        for nm_item in meta_list.nested {
            if let syn::NestedMeta::Meta(ref item) = nm_item {
                if let syn::Meta::NameValue(ref pair) = item {
                    match pair.ident.to_string().as_ref() {
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

        let source = source.expect("template path or source not found in attributes");
        match (&source, ext.is_some()) {
            (&Source::Path(_), true) => {
                panic!("'ext' attribute cannot be used with 'path' attribute")
            }
            (&Source::Source(_), false) => {
                panic!("must include 'ext' attribute when using 'source' attribute")
            }
            _ => {}
        }

        let escaping = match escaping {
            Some(m) => m,
            None => {
                let ext = match source {
                    Source::Path(ref p) => Path::new(p)
                        .extension()
                        .map(|s| s.to_str().unwrap())
                        .unwrap_or(""),
                    Source::Source(_) => ext.as_ref().unwrap(), // Already panicked if None
                };
                if HTML_EXTENSIONS.contains(&ext) {
                    EscapeMode::Html
                } else {
                    EscapeMode::None
                }
            }
        };

        let parent = match ast.data {
            syn::Data::Struct(syn::DataStruct {
                fields: syn::Fields::Named(ref fields),
                ..
            }) => fields.named.iter().filter_map(|f| {
                f.ident
                    .as_ref()
                    .and_then(|name| if name == "_parent" { Some(&f.ty) } else { None })
            }),
            _ => panic!("derive(Template) only works for struct items"),
        }.next();

        let path = match source {
            Source::Source(_) => match ext {
                Some(ref v) => PathBuf::from(format!("{}.{}", ast.ident, v)),
                None => PathBuf::new(),
            },
            Source::Path(ref s) => path::find_template_from_path(s, None),
        };

        TemplateInput {
            ast,
            source,
            print,
            escaping,
            ext,
            parent,
            path,
        }
    }
}

pub enum Source {
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
