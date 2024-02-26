use std::collections::hash_map::HashMap;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use mime::Mime;

use proc_macro::token_stream::IntoIter as ProcIter;
use proc_macro::{Delimiter, Literal, TokenStream, TokenTree};

use crate::config::{get_template_source, read_config_file, Config};
use crate::unescape::get_str_literal;
use crate::CompileError;
use parser::{Node, Parsed, Syntax};

pub(crate) struct TemplateInput<'a> {
    pub(crate) ast: &'a crate::DeriveInput,
    pub(crate) config: &'a Config<'a>,
    pub(crate) syntax: &'a Syntax<'a>,
    pub(crate) source: &'a Source,
    pub(crate) print: Print,
    pub(crate) escaper: &'a str,
    pub(crate) ext: Option<&'a str>,
    pub(crate) mime_type: String,
    pub(crate) path: PathBuf,
}

impl TemplateInput<'_> {
    /// Extract the template metadata from the `DeriveInput` structure. This
    /// mostly recovers the data for the `TemplateInput` fields from the
    /// `template()` attribute list fields.
    pub(crate) fn new<'n>(
        ast: &'n crate::DeriveInput,
        config: &'n Config<'_>,
        args: &'n TemplateArgs,
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
        let source = source
            .as_ref()
            .expect("template path or source not found in attributes");
        let path = match (&source, &ext) {
            (Source::Path(path), _) => config.find_template(path, None)?,
            (&Source::Source(_), Some(ext)) => PathBuf::from(format!("{}.{}", ast.ident, ext)),
            (&Source::Source(_), None) => {
                return Err("must include 'ext' attribute when using 'source' attribute".into())
            }
        };

        // Validate syntax
        let syntax = syntax.as_deref().map_or_else(
            || Ok(config.syntaxes.get(config.default_syntax).unwrap()),
            |s| {
                config
                    .syntaxes
                    .get(s)
                    .ok_or_else(|| CompileError::from(format!("attribute syntax {s} not exist")))
            },
        )?;

        // Match extension against defined output formats

        let escaping = escaping
            .as_deref()
            .unwrap_or_else(|| path.extension().map(|s| s.to_str().unwrap()).unwrap_or(""));

        let mut escaper = None;
        for (extensions, path) in &config.escapers {
            if extensions.contains(escaping) {
                escaper = Some(path);
                break;
            }
        }

        let escaper = escaper.ok_or_else(|| {
            CompileError::from(format!("no escaper defined for extension '{escaping}'"))
        })?;

        let mime_type =
            extension_to_mime_type(ext_default_to_path(ext.as_deref(), &path).unwrap_or("txt"))
                .to_string();

        Ok(TemplateInput {
            ast,
            config,
            syntax,
            source,
            print: *print,
            escaper,
            ext: ext.as_deref(),
            mime_type,
            path,
        })
    }

    pub(crate) fn find_used_templates(
        &self,
        map: &mut HashMap<PathBuf, Parsed>,
    ) -> Result<(), CompileError> {
        let source = match &self.source {
            Source::Source(s) => s.clone(),
            Source::Path(_) => get_template_source(&self.path)?,
        };

        let mut dependency_graph = Vec::new();
        let mut check = vec![(self.path.clone(), source)];
        while let Some((path, source)) = check.pop() {
            let parsed = Parsed::new(source, self.syntax)?;

            let mut top = true;
            let mut nested = vec![parsed.nodes()];
            while let Some(nodes) = nested.pop() {
                for n in nodes {
                    let mut add_to_check = |path: PathBuf| -> Result<(), CompileError> {
                        if !map.contains_key(&path) {
                            // Add a dummy entry to `map` in order to prevent adding `path`
                            // multiple times to `check`.
                            map.insert(path.clone(), Parsed::default());
                            let source = get_template_source(&path)?;
                            check.push((path, source));
                        }
                        Ok(())
                    };

                    use Node::*;
                    match n {
                        Extends(extends) if top => {
                            let extends = self.config.find_template(extends.path, Some(&path))?;
                            let dependency_path = (path.clone(), extends.clone());
                            if dependency_graph.contains(&dependency_path) {
                                return Err(format!(
                                    "cyclic dependency in graph {:#?}",
                                    dependency_graph
                                        .iter()
                                        .map(|e| format!("{:#?} --> {:#?}", e.0, e.1))
                                        .collect::<Vec<String>>()
                                )
                                .into());
                            }
                            dependency_graph.push(dependency_path);
                            add_to_check(extends)?;
                        }
                        Macro(m) if top => {
                            nested.push(&m.nodes);
                        }
                        Import(import) if top => {
                            let import = self.config.find_template(import.path, Some(&path))?;
                            add_to_check(import)?;
                        }
                        Include(include) => {
                            let include = self.config.find_template(include.path, Some(&path))?;
                            add_to_check(include)?;
                        }
                        BlockDef(b) => {
                            nested.push(&b.nodes);
                        }
                        If(i) => {
                            for cond in &i.branches {
                                nested.push(&cond.nodes);
                            }
                        }
                        Loop(l) => {
                            nested.push(&l.body);
                            nested.push(&l.else_nodes);
                        }
                        Match(m) => {
                            for arm in &m.arms {
                                nested.push(&arm.nodes);
                            }
                        }
                        Lit(_)
                        | Comment(_)
                        | Expr(_, _)
                        | Call(_)
                        | Extends(_)
                        | Let(_)
                        | Import(_)
                        | Macro(_)
                        | Raw(_)
                        | Continue(_)
                        | Break(_) => {}
                    }
                }
                top = false;
            }
            map.insert(path, parsed);
        }
        Ok(())
    }

    #[inline]
    pub(crate) fn extension(&self) -> Option<&str> {
        ext_default_to_path(self.ext, &self.path)
    }
}

struct AttributeIterator {
    inner: ProcIter,
    expecting_comma: bool,
}

impl AttributeIterator {
    fn new(stream: TokenStream) -> Self {
        Self {
            inner: stream.into_iter(),
            expecting_comma: false,
        }
    }
}

fn extract_literal(token: TokenTree) -> Result<Literal, CompileError> {
    match token {
        TokenTree::Literal(lit) => Ok(lit),
        // In case we are in a macro and the literal is a token, it might be in a `Group` with no
        // delimiter.
        TokenTree::Group(g) if g.delimiter() == Delimiter::None => {
            let Some(token) = g.stream().into_iter().next() else {
                return Err("expected literal after `=`".into());
            };
            extract_literal(token)
        }
        token => Err(format!("expected literal after `=`, found {token}").into()),
    }
}

impl Iterator for AttributeIterator {
    type Item = Result<(String, Literal), CompileError>;

    fn next(&mut self) -> Option<Self::Item> {
        // All attributes handled by `template` derive proc are in the same format:
        //  `<ident> = <literal>`.
        //
        // If anything is different, we error.
        let ident = match self.inner.next()? {
            TokenTree::Ident(ident) if !self.expecting_comma => ident,
            TokenTree::Punct(p) if self.expecting_comma && p.as_char() == ',' => {
                self.expecting_comma = false;
                return self.next();
            }
            token => {
                return Some(Err(format!("unexpected token `{token}`").into()));
            }
        };
        let punct = self
            .inner
            .next()
            .ok_or_else(|| format!("expected `=` after `{ident}`").into())
            .and_then(|token| match token {
                TokenTree::Punct(p) if p.as_char() == '=' => Ok(()),
                _ => Err(format!("expected `=` after `{ident}`, found {token}").into()),
            });
        if let Err(err) = punct {
            return Some(Err(err));
        }
        let literal = self
            .inner
            .next()
            .ok_or_else(|| "expected literated after `=`".into())
            .and_then(extract_literal);
        match literal {
            Ok(literal) => {
                self.expecting_comma = true;
                // Happy path complete!
                Some(Ok((ident.to_string(), literal)))
            }
            Err(err) => Some(Err(err)),
        }
    }
}

#[derive(Debug, Default)]
pub(crate) struct TemplateArgs {
    source: Option<Source>,
    print: Print,
    escaping: Option<String>,
    ext: Option<String>,
    syntax: Option<String>,
    config: Option<String>,
    pub(crate) whitespace: Option<String>,
}

impl TemplateArgs {
    pub(crate) fn new(ast: &'_ crate::DeriveInput) -> Result<Self, CompileError> {
        // Check that an attribute called `template()` exists once and that it is
        // the proper type (list).
        let mut template_args = None;

        for token_stream in &ast.attrs {
            let mut stream_iter = token_stream.clone().into_iter();
            while let Some(attr) = stream_iter.next() {
                if matches!(attr, TokenTree::Ident(i) if i.to_string() == "template") {
                    match stream_iter.next() {
                        Some(TokenTree::Group(g)) => {
                            if g.delimiter() == Delimiter::Parenthesis {
                                if template_args.is_none() {
                                    template_args = Some(g.stream());
                                } else {
                                    return Err("duplicated 'template' attribute".into());
                                }
                            } else {
                                let extra = match g.delimiter() {
                                    Delimiter::Brace => ", found `{`",
                                    Delimiter::Bracket => ", found `[`",
                                    _ => "",
                                };
                                return Err(format!("expected `(` after `template`{extra}").into());
                            }
                        }
                        Some(token) => {
                            return Err(
                                format!("expected `(` after `template`, found `{token}`").into()
                            )
                        }
                        None => return Err("expected `(` after `template`, found nothing".into()),
                    }
                }
            }
        }

        let template_args =
            template_args.ok_or_else(|| CompileError::from("no attribute 'template' found"))?;

        let mut args = Self::default();
        // Loop over the meta attributes and find everything that we
        // understand. Return a CompileError if something is not right.
        // `source` contains an enum that can represent `path` or `source`.
        for attribute in AttributeIterator::new(template_args) {
            let (ident, literal) = attribute?;

            if ident == "path" {
                let lit = get_str_literal(literal, &ident)?;
                if args.source.is_some() {
                    return Err("must specify 'source' or 'path', not both".into());
                }
                args.source = Some(Source::Path(lit));
            } else if ident == "source" {
                let lit = get_str_literal(literal, &ident)?;
                if args.source.is_some() {
                    return Err("must specify 'source' or 'path', not both".into());
                }
                args.source = Some(Source::Source(lit));
            } else if ident == "print" {
                let lit = get_str_literal(literal, &ident)?;
                args.print = lit.parse()?;
            } else if ident == "escape" {
                let lit = get_str_literal(literal, &ident)?;
                args.escaping = Some(lit);
            } else if ident == "ext" {
                let lit = get_str_literal(literal, &ident)?;
                args.ext = Some(lit);
            } else if ident == "syntax" {
                let lit = get_str_literal(literal, &ident)?;
                args.syntax = Some(lit)
            } else if ident == "config" {
                let lit = get_str_literal(literal, &ident)?;
                args.config = Some(lit);
            } else if ident == "whitespace" {
                let lit = get_str_literal(literal, &ident)?;
                args.whitespace = Some(lit)
            } else {
                return Err(format!("unsupported attribute key {ident:?} found").into());
            }
        }

        Ok(args)
    }

    pub(crate) fn config(&self) -> Result<String, CompileError> {
        read_config_file(self.config.as_deref())
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

#[derive(Debug)]
pub(crate) enum Source {
    Path(String),
    Source(String),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Print {
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
            v => return Err(format!("invalid value for print option: {v}",).into()),
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

const TEXT_TYPES: [(Mime, Mime); 7] = [
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
    (mime::IMAGE_SVG, mime::IMAGE_SVG),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ext() {
        assert_eq!(extension(Path::new("foo-bar.txt")), Some("txt"));
        assert_eq!(extension(Path::new("foo-bar.html")), Some("html"));
        assert_eq!(extension(Path::new("foo-bar.unknown")), Some("unknown"));
        assert_eq!(extension(Path::new("foo-bar.svg")), Some("svg"));

        assert_eq!(extension(Path::new("foo/bar/baz.txt")), Some("txt"));
        assert_eq!(extension(Path::new("foo/bar/baz.html")), Some("html"));
        assert_eq!(extension(Path::new("foo/bar/baz.unknown")), Some("unknown"));
        assert_eq!(extension(Path::new("foo/bar/baz.svg")), Some("svg"));
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
