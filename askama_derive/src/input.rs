use proc_macro2::TokenStream;

use quote::ToTokens;

use shared::path;

use std::path::PathBuf;

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
        let path = match (&source, &ext) {
            (&Source::Path(ref path), None) => path::find_template_from_path(path, None),
            (&Source::Source(_), Some(ext)) => PathBuf::from(format!("{}.{}", ast.ident, ext)),
            (&Source::Path(_), Some(_)) => {
                panic!("'ext' attribute cannot be used with 'path' attribute")
            }
            (&Source::Source(_), None) => {
                panic!("must include 'ext' attribute when using 'source' attribute")
            }
        };

        let parent = match ast.data {
            syn::Data::Struct(syn::DataStruct {
                fields: syn::Fields::Named(ref fields),
                ..
            }) => fields.named.iter().find(|f| {
                if let Some(field_name) = f.ident.as_ref() {
                    field_name == "_parent"
                } else {
                    false
                }
            }),
            _ => panic!("derive(Template) only works for struct items"),
        };
        let parent = if let Some(parent) = parent {
            Some(&parent.ty)
        } else {
            None
        };

        TemplateInput {
            ast,
            source,
            print,
            escaping: escaping.unwrap_or_else(|| EscapeMode::from_path(&path)),
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

impl EscapeMode {
    fn from_path(path: &PathBuf) -> EscapeMode {
        let extension = path.extension().map(|s| s.to_str().unwrap()).unwrap_or("");
        if HTML_EXTENSIONS.contains(&extension) {
            EscapeMode::Html
        } else {
            EscapeMode::None
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
