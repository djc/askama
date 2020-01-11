use std::path::PathBuf;

use darling::FromDeriveInput;
use darling::FromMeta;
use syn;

use askama_shared::{Config, Syntax};

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
}

#[derive(FromDeriveInput, Default)]
#[darling(attributes(template), default)]
struct TemplateInputParser {
    path: Option<String>,
    source: Option<String>,
    print: Print,
    escape: Option<String>,
    ext: Option<String>,
    syntax: Option<String>,
}

impl<'a> TemplateInput<'a> {
    /// Extract the template metadata from the `DeriveInput` structure. This
    /// mostly recovers the data for the `TemplateInput` fields from the
    /// `template()` attribute list fields; it also finds the of the `_parent`
    /// field, if any.
    pub fn new<'n>(ast: &'n syn::DeriveInput, config: &'n Config) -> TemplateInput<'n> {
        // Check that an attribute called `template()` exists and that it is
        // the proper type (list).
        let TemplateInputParser {
            path,
            source,
            print,
            escape,
            ext,
            syntax,
        } = FromDeriveInput::from_derive_input(ast)
            .expect("attribute 'template' not found or with wrong format");

        assert_ne!(
            source.is_some(),
            path.is_some(),
            "One of path or source must exist and they are mutually exclusive",
        );
        assert_eq!(
            source.is_some(),
            ext.is_some(),
            "must include 'ext' attribute when using 'source' attribute"
        );

        // Since 'source' and 'path' are related. In case `source` was used instead
        // of `path`, the value of `ext` is merged into a synthetic `path` value here.
        let source = source
            .map(Source::Source)
            .unwrap_or_else(|| Source::Path(path.unwrap()));
        let path = match (&source, &ext) {
            (&Source::Path(ref path), None) => config.find_template(path, None),
            (&Source::Source(_), Some(ext)) => PathBuf::from(format!("{}.{}", ast.ident, ext)),
            _ => unreachable!(),
        };

        // Check to see if a `_parent` field was defined on the context
        // struct, and store the type for it for use in the code generator.
        let parent = match ast.data {
            syn::Data::Struct(syn::DataStruct {
                fields: syn::Fields::Named(ref fields),
                ..
            }) => fields
                .named
                .iter()
                .find(|f| f.ident.as_ref().filter(|name| *name == "_parent").is_some())
                .map(|f| &f.ty),
            _ => None,
        };

        if parent.is_some() {
            eprint!(
                "   --> in struct {}\n   = use of deprecated field '_parent'\n",
                ast.ident
            );
        }

        // Validate syntax
        let syntax = syntax.map_or_else(
            || config.syntaxes.get(config.default_syntax).unwrap(),
            |s| {
                config
                    .syntaxes
                    .get(&s)
                    .unwrap_or_else(|| panic!("attribute syntax {} not exist", s))
            },
        );

        // Match extension against defined output formats
        let extension = escape.unwrap_or_else(|| {
            path.extension()
                .map(|s| s.to_str().unwrap())
                .unwrap_or("")
                .to_string()
        });

        let escaper = config
            .escapers
            .iter()
            .find(|(extensions, _)| extensions.contains(&extension))
            .as_ref()
            .map(|x| &x.1)
            .unwrap_or_else(|| panic!("no escaper defined for extension '{}'", extension));

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
        }
    }
}

pub enum Source {
    Path(String),
    Source(String),
}

#[derive(PartialEq, FromMeta)]
#[darling(rename_all = "lowercase")]
pub enum Print {
    All,
    Ast,
    Code,
    None,
}

impl Default for Print {
    fn default() -> Self {
        Self::None
    }
}
