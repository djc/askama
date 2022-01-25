use crate::parser::{one_expr, one_target};
use crate::{CompileError, Config, Syntax};

use std::path::{Path, PathBuf};
use std::str::FromStr;

use mime::Mime;
use quote::ToTokens;

#[derive(Clone)]
pub struct MultiTemplateInput<'a> {
    pub syntax: &'a Syntax<'a>,
    pub pattern: String,
    pub path: PathBuf,
    pub source: Source,
    pub escaper: &'a str,
}

pub struct TemplateInput<'a> {
    pub ast: &'a syn::DeriveInput,
    pub config: &'a Config<'a>,
    pub syntax: &'a Syntax<'a>,
    pub source: Source,
    pub print: Print,
    pub escaper: &'a str,
    pub ext: Option<String>,
    pub mime_type: String,
    pub path: PathBuf,
    pub multi: Option<(String, Vec<MultiTemplateInput<'a>>)>,
}

impl TemplateInput<'_> {
    /// Extract the template metadata from the `DeriveInput` structure. This
    /// mostly recovers the data for the `TemplateInput` fields from the
    /// `template()` attribute list fields; it also finds the of the `_parent`
    /// field, if any.
    pub fn new<'n>(
        ast: &'n syn::DeriveInput,
        config: &'n Config<'_>,
    ) -> Result<TemplateInput<'n>, CompileError> {
        // Check that an attribute called `template()` exists once and that it is
        // the proper type (list).
        let mut template_args = None;
        let mut multi_args = vec![];
        for attr in &ast.attrs {
            if attr.path.is_ident("template") {
                if template_args.is_some() {
                    return Err(CompileError::Static("duplicated 'template' attribute"));
                }

                match attr.parse_meta() {
                    Ok(syn::Meta::List(syn::MetaList { nested, .. })) => {
                        template_args = Some(nested);
                    }
                    Ok(_) => return Err("'template' attribute must be a list".into()),
                    Err(e) => return Err(format!("unable to parse attribute: {}", e).into()),
                }
            } else if attr.path.is_ident("multi_template") {
                match attr.parse_meta() {
                    Ok(syn::Meta::List(inner)) => {
                        multi_args.push(inner.nested);
                    }
                    Ok(_) => return Err("attribute 'multi_template' has incorrect type".into()),
                    Err(e) => {
                        return Err(
                            format!("unable to parse 'multi_template' attribute: {}", e).into()
                        )
                    }
                }
            }
        }
        let template_args =
            template_args.ok_or(CompileError::Static("no attribute 'template' found"))?;

        // Loop over the meta attributes and find everything that we
        // understand. Raise panics if something is not right.
        // `source` contains an enum that can represent `path` or `source`.
        let mut source = None;
        let mut path = None;
        let mut print = Print::None;
        let mut escaping = None;
        let mut ext = None;
        let mut syntax = None;
        let mut multi = None;

        for item in template_args {
            let pair = match item {
                syn::NestedMeta::Meta(syn::Meta::NameValue(ref pair)) => pair,
                _ => {
                    return Err(format!(
                        "unsupported attribute argument {:?}",
                        item.to_token_stream()
                    )
                    .into())
                }
            };

            if pair.path.is_ident("path") {
                if let syn::Lit::Str(ref s) = pair.lit {
                    path = Some(s.value());
                } else {
                    return Err("template path must be string literal".into());
                }
            } else if pair.path.is_ident("source") {
                if let syn::Lit::Str(ref s) = pair.lit {
                    source = Some(s.value());
                } else {
                    return Err("template source must be string literal".into());
                }
            } else if pair.path.is_ident("print") {
                if let syn::Lit::Str(ref s) = pair.lit {
                    print = s.value().parse()?;
                } else {
                    return Err("print value must be string literal".into());
                }
            } else if pair.path.is_ident("escape") {
                if let syn::Lit::Str(ref s) = pair.lit {
                    escaping = Some(s.value());
                } else {
                    return Err("escape value must be string literal".into());
                }
            } else if pair.path.is_ident("ext") {
                if let syn::Lit::Str(ref s) = pair.lit {
                    ext = Some(s.value());
                } else {
                    return Err("ext value must be string literal".into());
                }
            } else if pair.path.is_ident("syntax") {
                if let syn::Lit::Str(ref s) = pair.lit {
                    syntax = Some(s.value())
                } else {
                    return Err("syntax value must be string literal".into());
                }
            } else if pair.path.is_ident("multi") {
                if let syn::Lit::Str(ref s) = pair.lit {
                    let s = s.value();
                    if one_expr(&s).is_err() {
                        return Err(CompileError::Static("multi value must be expression"));
                    }
                    multi = Some(s);
                } else {
                    return Err("localizer value must be string literal".into());
                }
            } else {
                return Err(format!(
                    "unsupported attribute key '{}' found",
                    pair.path.to_token_stream()
                )
                .into());
            }
        }

        // 'multi' and 'multi_template' go together
        let multi = match (multi, multi_args) {
            (None, multi_args) if multi_args.is_empty() => None,
            (Some(localizer), multi_args) if !multi_args.is_empty() => {
                let multi = multi_args
                    .into_iter()
                    .map(|args| parse_multi(ast, config, args))
                    .collect::<Result<_, CompileError>>()?;
                Some((localizer, multi))
            }
            _ => {
                return Err(CompileError::Static(
                    "#[template(multi)] and #[multi_template] have to be used together",
                ));
            }
        };

        let (source, path) = select_source_and_path(ast, config, source, path, ext.as_deref())?;
        let escaper = select_escaper(config, escaping.as_deref(), &path)?;
        let syntax = select_syntax(config, syntax)?;

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
            multi,
        })
    }

    #[inline]
    pub fn extension(&self) -> Option<&str> {
        ext_default_to_path(self.ext.as_deref(), &self.path)
    }
}

fn parse_multi<'a>(
    ast: &syn::DeriveInput,
    config: &'a Config<'a>,
    args: impl IntoIterator<Item = syn::NestedMeta>,
) -> Result<MultiTemplateInput<'a>, CompileError> {
    let mut path = None;
    let mut source = None;
    let mut ext = None;
    let mut syntax = None;
    let mut escaping = None;
    let mut pattern = None;

    for item in args {
        let pair = match item {
            syn::NestedMeta::Meta(syn::Meta::NameValue(ref pair)) => pair,
            _ => {
                return Err(format!(
                    "unsupported attribute argument {:?}",
                    item.to_token_stream()
                )
                .into())
            }
        };

        if pair.path.is_ident("path") {
            if let syn::Lit::Str(ref s) = pair.lit {
                path = Some(s.value());
            } else {
                return Err("path value must be string literal".into());
            }
        } else if pair.path.is_ident("source") {
            if let syn::Lit::Str(ref s) = pair.lit {
                source = Some(s.value());
            } else {
                return Err("source value must be string literal".into());
            }
        } else if pair.path.is_ident("ext") {
            if let syn::Lit::Str(ref s) = pair.lit {
                ext = Some(s.value());
            } else {
                return Err("ext value must be string literal".into());
            }
        } else if pair.path.is_ident("syntax") {
            if let syn::Lit::Str(ref s) = pair.lit {
                syntax = Some(s.value());
            } else {
                return Err("pattern value must be string literal".into());
            }
        } else if pair.path.is_ident("escaping") {
            if let syn::Lit::Str(ref s) = pair.lit {
                escaping = Some(s.value());
            } else {
                return Err("escaping value must be string literal".into());
            }
        } else if pair.path.is_ident("pattern") {
            if let syn::Lit::Str(ref s) = pair.lit {
                pattern = Some(s.value());
            } else {
                return Err("pattern value must be string literal".into());
            }
        } else {
            return Err(format!(
                "unsupported attribute key '{}' found",
                pair.path.to_token_stream()
            )
            .into());
        }
    }

    let (source, path) = select_source_and_path(ast, config, source, path, ext.as_deref())?;
    let escaper = select_escaper(config, escaping.as_deref(), &path)?;
    let syntax = select_syntax(config, syntax)?;

    let pattern = pattern.ok_or(CompileError::Static("multi-template pattern missing"))?;
    if one_target(&pattern).is_err() {
        return Err("the pattern attribute could not be parsed".into());
    }

    Ok(MultiTemplateInput {
        pattern,
        source,
        path,
        escaper,
        syntax,
    })
}

fn select_syntax<'a>(
    config: &'a Config<'a>,
    syntax: Option<String>,
) -> Result<&'a Syntax<'a>, CompileError> {
    syntax.map_or_else(
        || Ok(config.syntaxes.get(config.default_syntax).unwrap()),
        |s| {
            config
                .syntaxes
                .get(&s)
                .ok_or_else(|| CompileError::String(format!("attribute syntax {} not exist", s)))
        },
    )
}

fn select_source_and_path(
    ast: &syn::DeriveInput,
    config: &Config<'_>,
    source: Option<String>,
    path: Option<String>,
    ext: Option<&str>,
) -> Result<(Source, PathBuf), CompileError> {
    // Validate the `source` and `ext` value together, since they are
    // related. In case `source` was used instead of `path`, the value
    // of `ext` is merged into a synthetic `path` value here.

    let source = match (source, path) {
        (None, None) => return Err("template path or source not found in attributes".into()),
        (Some(_), Some(_)) => return Err("must specify 'source' or 'path', not both".into()),
        (None, Some(path)) => Source::Path(path),
        (Some(source), None) => Source::Source(source),
    };

    let path = match (&source, ext) {
        (Source::Path(path), _) => config.find_template(path, None)?,
        (Source::Source(_), Some(ext)) => PathBuf::from(format!("{}.{}", ast.ident, ext)),
        (Source::Source(_), None) => {
            return Err("must include 'ext' attribute when using 'source' attribute".into())
        }
    };

    Ok((source, path))
}

fn select_escaper<'a>(
    config: &'a Config<'_>,
    escaping: Option<&str>,
    path: &Path,
) -> Result<&'a str, CompileError> {
    let escaping = escaping.unwrap_or_else(|| path.extension().map_or("", |s| s.to_str().unwrap()));

    let mut escaper = None;
    for (extensions, path) in &config.escapers {
        if extensions.contains(escaping) {
            escaper = Some(path);
            break;
        }
    }

    escaper.map_or_else(
        || {
            Err(CompileError::String(format!(
                "no escaper defined for extension '{}'",
                escaping
            )))
        },
        |s| Ok(s.as_str()),
    )
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

#[derive(Clone)]
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
