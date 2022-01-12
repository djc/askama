use crate::parser::{one_expr, one_target};
use crate::{CompileError, Config, Syntax};

use std::path::{Path, PathBuf};
use std::str::FromStr;

use mime::Mime;
use quote::ToTokens;

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
    pub l10n: Option<(String, Vec<(String, PathBuf)>)>, // TODO: rename
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
        let mut l10n = None;
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
            } else if attr.path.is_ident("l10n") {
                match attr.parse_meta() {
                    Ok(syn::Meta::List(inner)) => {
                        l10n.get_or_insert_with(Vec::new).push(inner.nested);
                    }
                    Ok(_) => return Err("attribute 'l10n' has incorrect type".into()),
                    Err(e) => return Err(format!("unable to parse 'l10n' attribute: {}", e).into()),
                }
            }
        }
        let template_args =
            template_args.ok_or(CompileError::Static("no attribute 'template' found"))?;

        // Loop over the meta attributes and find everything that we
        // understand. Raise panics if something is not right.
        // `source` contains an enum that can represent `path` or `source`.
        let mut source = None;
        let mut print = Print::None;
        let mut escaping = None;
        let mut ext = None;
        let mut syntax = None;
        let mut localizer = None;
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
                    if source.is_some() {
                        return Err("must specify 'source' or 'path', not both".into());
                    }
                    source = Some(Source::Path(s.value()));
                } else {
                    return Err("template path must be string literal".into());
                }
            } else if pair.path.is_ident("source") {
                if let syn::Lit::Str(ref s) = pair.lit {
                    if source.is_some() {
                        return Err("must specify 'source' or 'path', not both".into());
                    }
                    source = Some(Source::Source(s.value()));
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
            } else if pair.path.is_ident("localizer") {
                if let syn::Lit::Str(ref s) = pair.lit {
                    let s = s.value();
                    if one_expr(&s).is_err() {
                        return Err(CompileError::Static("localizer value must be expression"));
                    }
                    localizer = Some(s);
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

        // 'localizer' and 'l10n' go together
        let l10n = match (localizer, l10n) {
            (None, None) => None,
            (Some(localizer), Some(l10n)) => {
                let multi = l10n
                    .into_iter()
                    .map(|args| {
                        let mut multi_path = None;
                        let mut multi_pattern = None;
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
                                    multi_path = Some(s.value());
                                } else {
                                    return Err("path value must be string literal".into());
                                }
                            } else if pair.path.is_ident("pattern") {
                                if let syn::Lit::Str(ref s) = pair.lit {
                                    multi_pattern = Some(s.value());
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

                        // TODO: start_at = path ?
                        let multi_path = multi_path
                            .ok_or(CompileError::Static("multi-template path missing"))?;
                        let multi_path = config.find_template(&multi_path, None)?;

                        let multi_pattern = multi_pattern
                            .ok_or(CompileError::Static("multi-template pattern missing"))?;
                        if one_target(&multi_pattern).is_err() {
                            return Err("the pattern attribute could not be parsed".into());
                        }

                        Ok((multi_pattern, multi_path))
                    })
                    .collect::<Result<_, CompileError>>()?;
                Some((localizer, multi))
            }
            _ => {
                return Err(CompileError::Static(
                    "#[template(localizer)] and #[l10n] have to be used together",
                ));
            }
        };

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
            CompileError::String(format!("no escaper defined for extension '{}'", escaping,))
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
            parent,
            path,
            l10n,
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
