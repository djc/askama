//! Input for the `template()` attribute.

use crate::config::{Config, Syntax};
use crate::generator::TemplateArgs;
use crate::CompileError;

use std::path::{Path, PathBuf};
use std::str::FromStr;

use mime::Mime;

/// Input configuration passed to the Askama `template()` attribute.
pub struct TemplateInput<'a> {
    /// The original raw `syn::DeriveInput` parsed.
    pub ast: &'a syn::DeriveInput,
    /// The configuration in use.
    pub config: &'a Config,
    /// The syntax used for this template.
    pub syntax: &'a Syntax,
    /// The source of the template.
    pub source: Source,
    /// Debug printing mode.
    pub print: Print,
    /// The escaper to use for the template.
    pub escaper: &'a str,
    /// The file extension specified for inline templates.
    pub ext: Option<String>,
    /// The MIME type of the template's results.
    pub mime_type: String,
    /// The generated path of the template.
    pub path: PathBuf,
}

impl TemplateInput<'_> {
    /// Extract the template metadata from the `DeriveInput` structure. This
    /// mostly recovers the data for the `TemplateInput` fields from the
    /// `template()` attribute list fields.
    pub fn new<'n>(
        ast: &'n syn::DeriveInput,
        config: &'n Config,
        args: TemplateArgs,
    ) -> Result<TemplateInput<'n>, CompileError> {
        let TemplateArgs {
            source,
            print,
            escaping,
            ext,
            syntax,
            ..
        } = args;

        // Validate the `source` and `ext` value together, since they are
        // related. In case `source` was used instead of `path`, the value
        // of `ext` is merged into a synthetic `path` value here.
        let source = source.expect("template path or source not found in attributes");
        let path = match (&source, &ext) {
            (&Source::Path(ref path), _) => config.find_template(path, None)?,
            (&Source::Source(_), Some(ext)) => PathBuf::from(format!("{}.{}", ast.ident, ext)),
            (&Source::Source(_), None) => {
                return Err("must include 'ext' attribute when using 'source' attribute".into())
            }
        };

        // Validate syntax
        let syntax = syntax.map_or_else(
            || Ok(config.syntaxes.get(&config.default_syntax).unwrap()),
            |s| {
                config
                    .syntaxes
                    .get(&s)
                    .ok_or_else(|| CompileError::from(format!("attribute syntax {} not exist", s)))
            },
        )?;

        // Match extension against defined output formats

        let escaping = escaping.unwrap_or_else(|| {
            path.extension()
                .map(|s| s.to_str().unwrap())
                .unwrap_or("")
                .to_string()
        });

        let mut escaper = None;
        for (extensions, path) in &config.escapers {
            if extensions.contains(&escaping) {
                escaper = Some(path);
                break;
            }
        }

        let escaper = escaper.ok_or_else(|| {
            CompileError::from(format!("no escaper defined for extension '{}'", escaping))
        })?;

        let mime_type =
            extension_to_mime_type(ext_default_to_path(ext.as_deref(), &path).unwrap_or("txt"))
                .to_string();

        Ok(TemplateInput {
            ast,
            config,
            syntax,
            source,
            print,
            escaper,
            ext,
            mime_type,
            path,
        })
    }

    #[inline]
    pub fn extension(&self) -> Option<&str> {
        ext_default_to_path(self.ext.as_deref(), &self.path)
    }
}

#[inline]
fn ext_default_to_path<'a>(ext: Option<&'a str>, path: &'a Path) -> Option<&'a str> {
    ext.or_else(|| extension(path))
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

/// Ways to specify the source for an Askama template.
pub enum Source {
    /// Load the specified template file.
    ///
    /// The path is interpreted as relative to the configured template directories
    /// (by default, this is a templates directory next to your Cargo.toml).
    Path(String),
    /// Directly set the template source.
    ///
    /// This can be useful for test cases or short templates. The generated path is
    /// undefined, which generally makes it impossible to refer to this template
    /// from other templates.
    Source(String),
}

/// Print debug information at compile time.
#[derive(PartialEq, Eq)]
#[non_exhaustive]
pub enum Print {
    /// Print all debug info.
    All,
    /// Print the parsed syntax tree.
    Ast,
    /// Print the generated code.
    Code,
    /// Do not print.
    None,
}

impl FromStr for Print {
    type Err = CompileError;

    fn from_str(s: &str) -> Result<Print, Self::Err> {
        use self::Print::*;
        Ok(match s {
            "all" => All,
            "ast" => Ast,
            "code" => Code,
            "none" => None,
            v => return Err(format!("invalid value for print option: {}", v,).into()),
        })
    }
}

impl Default for Print {
    fn default() -> Self {
        Self::None
    }
}

pub(crate) fn extension_to_mime_type(ext: &str) -> Mime {
    let basic_type = mime_guess::from_ext(ext).first_or_octet_stream();
    for (simple, utf_8) in &TEXT_TYPES {
        if &basic_type == simple {
            return utf_8.clone();
        }
    }
    basic_type
}

const TEXT_TYPES: [(Mime, Mime); 6] = [
    (mime::TEXT_PLAIN, mime::TEXT_PLAIN_UTF_8),
    (mime::TEXT_HTML, mime::TEXT_HTML_UTF_8),
    (mime::TEXT_CSS, mime::TEXT_CSS_UTF_8),
    (mime::TEXT_CSV, mime::TEXT_CSV_UTF_8),
    (
        mime::TEXT_TAB_SEPARATED_VALUES,
        mime::TEXT_TAB_SEPARATED_VALUES_UTF_8,
    ),
    (
        mime::APPLICATION_JAVASCRIPT,
        mime::APPLICATION_JAVASCRIPT_UTF_8,
    ),
];

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
