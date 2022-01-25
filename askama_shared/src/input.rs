use crate::{Config, Syntax};

use std::borrow::Cow;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use mime::Mime;
use proc_macro2::Span;
use syn::spanned::Spanned;

pub struct TemplateInput<'a> {
    pub ast: &'a syn::DeriveInput,
    pub config: &'a Config<'a>,
    pub syntax: &'a Syntax<'a>,
    pub source: Source,
    pub print: Print,
    pub escaper: &'a str,
    pub ext: Option<String>,
    pub mime_type: String,
    pub parent: Option<&'a syn::Type>,
    pub path: PathBuf,
}

impl TemplateInput<'_> {
    /// Extract the template metadata from the `DeriveInput` structure. This
    /// mostly recovers the data for the `TemplateInput` fields from the
    /// `template()` attribute list fields; it also finds the of the `_parent`
    /// field, if any.
    pub fn new<'n>(
        ast: &'n syn::DeriveInput,
        config: &'n Config<'_>,
    ) -> Result<TemplateInput<'n>, syn::Error> {
        // Check that an attribute called `template()` exists once and that it is
        // the proper type (list).
        let mut template_args = None;
        for attr in &ast.attrs {
            let ident = match attr.path.get_ident() {
                Some(ident) => ident,
                None => continue,
            };
            if ident == "template" {
                if template_args.is_some() {
                    return Err(syn::Error::new(
                        ident.span(),
                        "duplicated 'template' attribute",
                    ));
                }

                match attr.parse_meta() {
                    Ok(syn::Meta::List(syn::MetaList { nested, .. })) => {
                        template_args = Some((ident.span(), nested));
                    }
                    Ok(meta) => {
                        return Err(syn::Error::new(
                            meta.span(),
                            "'template' attribute must be a list",
                        ));
                    }
                    Err(e) => {
                        return Err(syn::Error::new(
                            e.span(),
                            format!("unable to parse attribute: {}", e),
                        ));
                    }
                }
            }
        }
        let (template_span, template_args) = template_args.ok_or_else(|| {
            syn::Error::new(
                proc_macro2::Span::call_site(),
                "no attribute 'template' found",
            )
        })?;

        // Loop over the meta attributes and find everything that we
        // understand. Raise panics if something is not right.
        // `source` contains an enum that can represent `path` or `source`.
        let mut source = None;
        let mut print = Print::None;
        let mut escaping = None;
        let mut ext = None;
        let mut syntax = None;
        for item in template_args {
            let pair = match item {
                syn::NestedMeta::Meta(syn::Meta::NameValue(pair)) => pair,
                meta => {
                    return Err(syn::Error::new(
                        meta.span(),
                        "unsupported attribute argument",
                    ));
                }
            };

            let ident = match pair.path.get_ident() {
                Some(ident) => ident,
                None => unreachable!("impossible in Meta::NameValue(…)"),
            };

            if ident == "path" {
                if let syn::Lit::Str(ref s) = pair.lit {
                    if source.is_some() {
                        return Err(syn::Error::new(
                            pair.lit.span(),
                            "must specify 'source' or 'path', not both",
                        ));
                    }
                    source = Some(Source::Path(s.value(), s.span()));
                } else {
                    return Err(syn::Error::new(
                        pair.lit.span(),
                        "template path must be string literal",
                    ));
                }
            } else if ident == "source" {
                if let syn::Lit::Str(ref s) = pair.lit {
                    if source.is_some() {
                        return Err(syn::Error::new(
                            pair.lit.span(),
                            "must specify 'source' or 'path', not both",
                        ));
                    }
                    source = Some(Source::Source(s.value(), s.span()));
                } else {
                    return Err(syn::Error::new(
                        pair.lit.span(),
                        "template source must be string literal",
                    ));
                }
            } else if ident == "print" {
                if let syn::Lit::Str(ref s) = pair.lit {
                    print = s
                        .value()
                        .parse()
                        .map_err(|msg| syn::Error::new(pair.lit.span(), msg))?;
                } else {
                    return Err(syn::Error::new(
                        pair.lit.span(),
                        "print value must be string literal",
                    ));
                }
            } else if ident == "escape" {
                if let syn::Lit::Str(ref s) = pair.lit {
                    escaping = Some(s.value());
                } else {
                    return Err(syn::Error::new(
                        pair.lit.span(),
                        "escape value must be string literal",
                    ));
                }
            } else if ident == "ext" {
                if let syn::Lit::Str(s) = pair.lit {
                    ext = Some(s);
                } else {
                    return Err(syn::Error::new(
                        pair.lit.span(),
                        "ext value must be string literal",
                    ));
                }
            } else if ident == "syntax" {
                if let syn::Lit::Str(s) = pair.lit {
                    syntax = Some(s);
                } else {
                    return Err(syn::Error::new(
                        pair.lit.span(),
                        "syntax value must be string literal",
                    ));
                }
            } else {
                return Err(syn::Error::new(
                    ident.span(),
                    "unsupported attribute key found",
                ));
            }
        }

        // Validate the `source` and `ext` value together, since they are
        // related. In case `source` was used instead of `path`, the value
        // of `ext` is merged into a synthetic `path` value here.
        let ext_value = ext.as_ref().map(|ext| ext.value());
        let source = source.expect("template path or source not found in attributes");
        let path = match (&source, &ext_value) {
            (&Source::Path(ref path, span), _) => config
                .find_template(path, None)
                .map_err(|msg| syn::Error::new(span, msg))?,
            (&Source::Source(_, _), Some(ext)) => PathBuf::from(format!("{}.{}", ast.ident, ext)),
            (&Source::Source(_, _), None) => {
                return Err(syn::Error::new(
                    template_span,
                    "must include 'ext' attribute when using 'source' attribute",
                ));
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
        let syntax = match syntax {
            Some(lit) => {
                let s = lit.value();
                match config.syntaxes.get(&s) {
                    Some(syntax) => syntax,
                    None => {
                        return Err(syn::Error::new(
                            lit.span(),
                            format!("attribute syntax {} not exist", s),
                        ));
                    }
                }
            }
            None => config.syntaxes.get(config.default_syntax).unwrap(),
        };

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
            let span = match (&ext, &source) {
                (Some(ext), _) => ext.span(),
                (None, Source::Path(_, span)) => *span,
                _ => template_span,
            };
            syn::Error::new(
                span,
                format!("no escaper defined for extension '{}'", escaping),
            )
        })?;

        let mime_type = extension_to_mime_type(
            ext_default_to_path(ext_value.as_deref(), &path).unwrap_or("txt"),
        )
        .to_string();

        Ok(TemplateInput {
            ast,
            config,
            syntax,
            source,
            print,
            escaper,
            ext: ext_value,
            mime_type,
            parent,
            path,
        })
    }

    #[inline]
    pub fn extension(&self) -> Option<&str> {
        ext_default_to_path(self.ext.as_deref(), &self.path)
    }
}

#[inline]
pub fn ext_default_to_path<'a>(ext: Option<&'a str>, path: &'a Path) -> Option<&'a str> {
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

pub enum Source {
    Path(String, Span),
    Source(String, Span),
}

#[derive(PartialEq)]
pub enum Print {
    All,
    Ast,
    Code,
    None,
}

impl FromStr for Print {
    type Err = Cow<'static, str>;

    fn from_str(s: &str) -> Result<Print, Self::Err> {
        use self::Print::*;
        Ok(match s {
            "all" => All,
            "ast" => Ast,
            "code" => Code,
            "none" => None,
            v => {
                return Err(format!("invalid value for print option: {}", v).into());
            }
        })
    }
}

#[doc(hidden)]
pub fn extension_to_mime_type(ext: &str) -> Mime {
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
