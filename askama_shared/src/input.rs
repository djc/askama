use crate::{CompileError, Config, Syntax};

use std::path::{Path, PathBuf};

use darling::{FromDeriveInput, FromMeta};

/// Helper struct for converting an AST into template options.
#[derive(Default, FromDeriveInput)]
#[darling(default, attributes(template))]
struct TemplateOptions {
    path: Option<String>,
    source: Option<String>,
    print: Print,
    escape: Option<String>,
    ext: Option<String>,
    syntax: Option<String>,
}

impl TemplateOptions {
    /// Get the template source, based on either the `path` or `source` meta-item.
    ///
    /// A deriving struct must set exactly one of these, so this function returns an
    /// error if both - or neither - are set.
    fn source(&self) -> darling::Result<Source> {
        if self.path.is_some() && self.source.is_some() {
            Err(darling::Error::custom(
                "Cannot specify both path and source",
            ))
        } else {
            self.path
                .clone()
                .map(Source::Path)
                .or_else(|| self.source.clone().map(Source::Source))
                .ok_or_else(|| darling::Error::custom("Either path or source is required"))
        }
    }
}

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

impl<'a> TemplateInput<'a> {
    /// Extract the template metadata from the `DeriveInput` structure. This
    /// mostly recovers the data for the `TemplateInput` fields from the
    /// `template()` attribute list fields; it also finds the of the `_parent`
    /// field, if any.
    pub fn new<'n>(
        ast: &'n syn::DeriveInput,
        config: &'n Config<'_>,
    ) -> Result<TemplateInput<'n>, CompileError> {
        let opts = TemplateOptions::from_derive_input(ast)?;
        let source = opts.source()?;
        let TemplateOptions {
            print,
            syntax,
            ext,
            escape: escaping,
            ..
        } = opts;

        // Validate the `source` and `ext` value together, since they are
        // related. In case `source` was used instead of `path`, the value
        // of `ext` is merged into a synthetic `path` value here.
        let path = match (&source, &ext) {
            (&Source::Path(ref path), _) => config.find_template(path, None)?,
            (&Source::Source(_), Some(ext)) => PathBuf::from(format!("{}.{}", ast.ident, ext)),
            (&Source::Source(_), None) => {
                return Err("must include 'ext' attribute when using 'source' attribute".into())
            }
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
            || Ok(config.syntaxes.get(config.default_syntax).unwrap()),
            |s| {
                config.syntaxes.get(&s).ok_or_else(|| {
                    CompileError::String(format!("attribute syntax {} not exist", s))
                })
            },
        )?;

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

        let escaper = escaper.ok_or_else(|| {
            CompileError::String(format!("no escaper defined for extension '{}'", extension,))
        })?;

        Ok(TemplateInput {
            ast,
            config,
            syntax,
            source,
            print,
            escaper,
            ext,
            parent,
            path,
        })
    }

    pub fn extension(&self) -> Option<&str> {
        self.ext.as_deref().or_else(|| extension(&self.path))
    }
}

fn extension(path: &Path) -> Option<&str> {
    let ext = path.extension().map(|s| s.to_str().unwrap())?;

    const JINJA_EXTENSIONS: [&str; 3] = ["j2", "jinja", "jinja2"];
    if JINJA_EXTENSIONS.contains(&ext) {
        Path::new(path.file_stem().unwrap())
            .extension()
            .map(|s| s.to_str().unwrap())
            .or(Some(ext))
    } else {
        Some(ext)
    }
}

#[derive(Clone)]
pub enum Source {
    Path(String),
    Source(String),
}

#[derive(PartialEq, FromMeta)]
pub enum Print {
    All,
    Ast,
    Code,
    None,
}

impl Default for Print {
    fn default() -> Self {
        Print::None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ext() {
        assert_eq!(extension(Path::new("foo-bar.txt")), Some("txt"));
        assert_eq!(extension(Path::new("foo-bar.html")), Some("html"));
        assert_eq!(extension(Path::new("foo-bar.unknown")), Some("unknown"));

        assert_eq!(extension(Path::new("foo/bar/baz.txt")), Some("txt"));
        assert_eq!(extension(Path::new("foo/bar/baz.html")), Some("html"));
        assert_eq!(extension(Path::new("foo/bar/baz.unknown")), Some("unknown"));
    }

    #[test]
    fn test_double_ext() {
        assert_eq!(extension(Path::new("foo-bar.html.txt")), Some("txt"));
        assert_eq!(extension(Path::new("foo-bar.txt.html")), Some("html"));
        assert_eq!(extension(Path::new("foo-bar.txt.unknown")), Some("unknown"));

        assert_eq!(extension(Path::new("foo/bar/baz.html.txt")), Some("txt"));
        assert_eq!(extension(Path::new("foo/bar/baz.txt.html")), Some("html"));
        assert_eq!(
            extension(Path::new("foo/bar/baz.txt.unknown")),
            Some("unknown")
        );
    }

    #[test]
    fn test_skip_jinja_ext() {
        assert_eq!(extension(Path::new("foo-bar.html.j2")), Some("html"));
        assert_eq!(extension(Path::new("foo-bar.html.jinja")), Some("html"));
        assert_eq!(extension(Path::new("foo-bar.html.jinja2")), Some("html"));

        assert_eq!(extension(Path::new("foo/bar/baz.txt.j2")), Some("txt"));
        assert_eq!(extension(Path::new("foo/bar/baz.txt.jinja")), Some("txt"));
        assert_eq!(extension(Path::new("foo/bar/baz.txt.jinja2")), Some("txt"));
    }

    #[test]
    fn test_only_jinja_ext() {
        assert_eq!(extension(Path::new("foo-bar.j2")), Some("j2"));
        assert_eq!(extension(Path::new("foo-bar.jinja")), Some("jinja"));
        assert_eq!(extension(Path::new("foo-bar.jinja2")), Some("jinja2"));
    }
}
