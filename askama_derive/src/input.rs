use proc_macro2::TokenStream;

use quote::ToTokens;

use askama_shared::{Config, Syntax};

use std::io::{self, Write};
use std::path::PathBuf;

use syn;

pub struct TemplateInput<'a> {
    pub ast: &'a syn::DeriveInput,
    pub config: &'a Config<'a>,
    pub syntax: &'a Syntax<'a>,
    pub source: Source,
    pub print: Print,
    pub escaper: &'a str,
    pub ext: Option<String>,
    pub parent: Option<&'a syn::Type>,
    pub path: PathBuf,
    pub localizer: Option<&'a syn::Ident>,
}

impl<'a> TemplateInput<'a> {
    /// Extract the template metadata from the `DeriveInput` structure. This
    /// mostly recovers the data for the `TemplateInput` fields from the
    /// `template()` attribute list fields; it also finds the of the `_parent`
    /// field, if any.
    pub fn new<'n>(ast: &'n syn::DeriveInput, config: &'n Config) -> TemplateInput<'n> {
        // Check that an attribute called `template()` exists and that it is
        // the proper type (list).
        let mut meta = None;
        for attr in &ast.attrs {
            match attr.interpret_meta() {
                Some(m) => {
                    if m.name() == "template" {
                        meta = Some(m)
                    }
                }
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

        // Loop over the meta attributes and find everything that we
        // understand. Raise panics if something is not right.
        // `source` contains an enum that can represent `path` or `source`.
        let mut source = None;
        let mut print = Print::None;
        let mut escaping = None;
        let mut ext = None;
        let mut syntax = None;
        for nm_item in meta_list.nested {
            if let syn::NestedMeta::Meta(ref item) = nm_item {
                if let syn::Meta::NameValue(ref pair) = item {
                    match pair.ident.to_string().as_ref() {
                        "path" => {
                            if let syn::Lit::Str(ref s) = pair.lit {
                                if source.is_some() {
                                    panic!("must specify 'source' or 'path', not both");
                                }
                                source = Some(Source::Path(s.value()));
                            } else {
                                panic!("template path must be string literal");
                            }
                        }
                        "source" => {
                            if let syn::Lit::Str(ref s) = pair.lit {
                                if source.is_some() {
                                    panic!("must specify 'source' or 'path', not both");
                                }
                                source = Some(Source::Source(s.value()));
                            } else {
                                panic!("template source must be string literal");
                            }
                        }
                        "print" => {
                            if let syn::Lit::Str(ref s) = pair.lit {
                                print = s.value().into();
                            } else {
                                panic!("print value must be string literal");
                            }
                        }
                        "escape" => {
                            if let syn::Lit::Str(ref s) = pair.lit {
                                escaping = Some(s.value());
                            } else {
                                panic!("escape value must be string literal");
                            }
                        }
                        "ext" => {
                            if let syn::Lit::Str(ref s) = pair.lit {
                                ext = Some(s.value());
                            } else {
                                panic!("ext value must be string literal");
                            }
                        }
                        "syntax" => {
                            if let syn::Lit::Str(ref s) = pair.lit {
                                syntax = Some(s.value())
                            } else {
                                panic!("syntax value must be string literal");
                            }
                        }
                        attr => panic!("unsupported annotation key '{}' found", attr),
                    }
                }
            }
        }

        // Validate the `source` and `ext` value together, since they are
        // related. In case `source` was used instead of `path`, the value
        // of `ext` is merged into a synthetic `path` value here.
        let source = source.expect("template path or source not found in attributes");
        let path = match (&source, &ext) {
            (&Source::Path(ref path), None) => config.find_template(path, None),
            (&Source::Source(_), Some(ext)) => PathBuf::from(format!("{}.{}", ast.ident, ext)),
            (&Source::Path(_), Some(_)) => {
                panic!("'ext' attribute cannot be used with 'path' attribute")
            }
            (&Source::Source(_), None) => {
                panic!("must include 'ext' attribute when using 'source' attribute")
            }
        };

        // Check to see if a `_parent` field was defined on the context
        // struct, and store the type for it for use in the code generator.
        let mut parent = None;
        let mut localizer = None;

        if let syn::Data::Struct(syn::DataStruct {
            fields: syn::Fields::Named(ref fields),
            ..
        }) = ast.data
        {
            for field in fields.named.iter() {
                if field.ident.is_none() {
                    // can't use this field
                    continue;
                }
                let ident = field.ident.as_ref().unwrap();
                if ident == "_parent" {
                    parent = Some(&field.ty);
                }
                for attr in field.attrs.iter() {
                    if attr.path.is_ident("localizer") {
                        if localizer.is_some() {
                            panic!("Can't have multiple localizers for a single template!");
                        }
                        localizer = Some(&attr.path.segments[0].ident)
                    }
                }
            }
        }

        if parent.is_some() {
            io::stderr()
                .write_fmt(format_args!(
                    "   --> in struct {}\n   = use of deprecated field '_parent'\n",
                    ast.ident
                ))
                .unwrap();
        }

        // Validate syntax
        let syntax = syntax.map_or_else(
            || config.syntaxes.get(config.default_syntax).unwrap(),
            |s| {
                config
                    .syntaxes
                    .get(&s)
                    .expect(&format!("attribute syntax {} not exist", s))
            },
        );

        // Match extension against defined output formats

        let extension = escaping.unwrap_or_else(|| {
            path.extension()
                .map(|s| s.to_str().unwrap())
                .unwrap_or("")
                .to_string()
        });

        let mut escaper = None;
        for (extensions, path) in &config.escapers {
            if extensions.contains(&extension) {
                escaper = Some(path);
                break;
            }
        }

        let escaper = escaper.unwrap_or_else(|| {
            panic!("no escaper defined for extension '{}'", extension);
        });

        TemplateInput {
            ast,
            config,
            source,
            print,
            escaper,
            ext,
            parent,
            path,
            syntax,
            localizer,
        }
    }
}

pub enum Source {
    Path(String),
    Source(String),
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
