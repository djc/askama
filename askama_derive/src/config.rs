use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};
use std::{env, fs};

#[cfg(feature = "serde")]
use serde::Deserialize;

use crate::CompileError;
use parser::node::Whitespace;
use parser::Syntax;

#[derive(Debug)]
pub(crate) struct Config<'a> {
    pub(crate) dirs: Vec<PathBuf>,
    pub(crate) syntaxes: BTreeMap<String, Syntax<'a>>,
    pub(crate) default_syntax: &'a str,
    pub(crate) escapers: Vec<(HashSet<String>, String)>,
    pub(crate) whitespace: WhitespaceHandling,
}

impl<'a> Config<'a> {
    pub(crate) fn new(
        s: &'a str,
        template_whitespace: Option<&str>,
    ) -> std::result::Result<Config<'a>, CompileError> {
        let root = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
        let default_dirs = vec![root.join("templates")];

        let mut syntaxes = BTreeMap::new();
        syntaxes.insert(DEFAULT_SYNTAX_NAME.to_string(), Syntax::default());

        let raw = if s.is_empty() {
            RawConfig::default()
        } else {
            RawConfig::from_toml_str(s)?
        };

        let (dirs, default_syntax, mut whitespace) = match raw.general {
            Some(General {
                dirs,
                default_syntax,
                whitespace,
            }) => (
                dirs.map_or(default_dirs, |v| {
                    v.into_iter().map(|dir| root.join(dir)).collect()
                }),
                default_syntax.unwrap_or(DEFAULT_SYNTAX_NAME),
                whitespace,
            ),
            None => (
                default_dirs,
                DEFAULT_SYNTAX_NAME,
                WhitespaceHandling::default(),
            ),
        };
        if let Some(template_whitespace) = template_whitespace {
            whitespace = match template_whitespace {
                "suppress" => WhitespaceHandling::Suppress,
                "minimize" => WhitespaceHandling::Minimize,
                "preserve" => WhitespaceHandling::Preserve,
                s => return Err(format!("invalid value for `whitespace`: \"{s}\"").into()),
            };
        }

        if let Some(raw_syntaxes) = raw.syntax {
            for raw_s in raw_syntaxes {
                let name = raw_s.name;

                if syntaxes
                    .insert(name.to_string(), raw_s.try_into()?)
                    .is_some()
                {
                    return Err(format!("syntax \"{name}\" is already defined").into());
                }
            }
        }

        if !syntaxes.contains_key(default_syntax) {
            return Err(format!("default syntax \"{default_syntax}\" not found").into());
        }

        let mut escapers = Vec::new();
        if let Some(configured) = raw.escaper {
            for escaper in configured {
                escapers.push((
                    escaper
                        .extensions
                        .iter()
                        .map(|ext| (*ext).to_string())
                        .collect(),
                    escaper.path.to_string(),
                ));
            }
        }
        for (extensions, path) in DEFAULT_ESCAPERS {
            escapers.push((str_set(extensions), (*path).to_string()));
        }

        Ok(Config {
            dirs,
            syntaxes,
            default_syntax,
            escapers,
            whitespace,
        })
    }

    pub(crate) fn find_template(
        &self,
        path: &str,
        start_at: Option<&Path>,
    ) -> std::result::Result<PathBuf, CompileError> {
        if let Some(root) = start_at {
            let relative = root.with_file_name(path);
            if relative.exists() {
                return Ok(relative);
            }
        }

        for dir in &self.dirs {
            let rooted = dir.join(path);
            if rooted.exists() {
                return Ok(rooted);
            }
        }

        Err(format!(
            "template {:?} not found in directories {:?}",
            path, self.dirs
        )
        .into())
    }
}

impl<'a> TryInto<Syntax<'a>> for RawSyntax<'a> {
    type Error = CompileError;

    fn try_into(self) -> Result<Syntax<'a>, Self::Error> {
        let default = Syntax::default();
        let syntax = Syntax {
            block_start: self.block_start.unwrap_or(default.block_start),
            block_end: self.block_end.unwrap_or(default.block_end),
            expr_start: self.expr_start.unwrap_or(default.expr_start),
            expr_end: self.expr_end.unwrap_or(default.expr_end),
            comment_start: self.comment_start.unwrap_or(default.comment_start),
            comment_end: self.comment_end.unwrap_or(default.comment_end),
        };

        for s in [
            syntax.block_start,
            syntax.block_end,
            syntax.expr_start,
            syntax.expr_end,
            syntax.comment_start,
            syntax.comment_end,
        ] {
            if s.len() < 2 {
                return Err(
                    format!("delimiters must be at least two characters long: {s:?}").into(),
                );
            } else if s.chars().any(|c| c.is_whitespace()) {
                return Err(format!("delimiters may not contain white spaces: {s:?}").into());
            }
        }

        for (s1, s2) in [
            (syntax.block_start, syntax.expr_start),
            (syntax.block_start, syntax.comment_start),
            (syntax.expr_start, syntax.comment_start),
        ] {
            if s1.starts_with(s2) || s2.starts_with(s1) {
                return Err(format!(
                    "a delimiter may not be the prefix of another delimiter: {s1:?} vs {s2:?}",
                )
                .into());
            }
        }

        Ok(syntax)
    }
}

#[cfg_attr(feature = "serde", derive(Deserialize))]
#[derive(Default)]
struct RawConfig<'a> {
    #[cfg_attr(feature = "serde", serde(borrow))]
    general: Option<General<'a>>,
    syntax: Option<Vec<RawSyntax<'a>>>,
    escaper: Option<Vec<RawEscaper<'a>>>,
}

impl RawConfig<'_> {
    #[cfg(feature = "config")]
    fn from_toml_str(s: &str) -> std::result::Result<RawConfig<'_>, CompileError> {
        basic_toml::from_str(s)
            .map_err(|e| format!("invalid TOML in {CONFIG_FILE_NAME}: {e}").into())
    }

    #[cfg(not(feature = "config"))]
    fn from_toml_str(_: &str) -> std::result::Result<RawConfig<'_>, CompileError> {
        Err("TOML support not available".into())
    }
}

#[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "serde", derive(Deserialize))]
#[cfg_attr(feature = "serde", serde(field_identifier, rename_all = "lowercase"))]
pub(crate) enum WhitespaceHandling {
    /// The default behaviour. It will leave the whitespace characters "as is".
    #[default]
    Preserve,
    /// It'll remove all the whitespace characters before and after the jinja block.
    Suppress,
    /// It'll remove all the whitespace characters except one before and after the jinja blocks.
    /// If there is a newline character, the preserved character in the trimmed characters, it will
    /// the one preserved.
    Minimize,
}

impl From<WhitespaceHandling> for Whitespace {
    fn from(ws: WhitespaceHandling) -> Self {
        match ws {
            WhitespaceHandling::Suppress => Whitespace::Suppress,
            WhitespaceHandling::Preserve => Whitespace::Preserve,
            WhitespaceHandling::Minimize => Whitespace::Minimize,
        }
    }
}

#[cfg_attr(feature = "serde", derive(Deserialize))]
struct General<'a> {
    #[cfg_attr(feature = "serde", serde(borrow))]
    dirs: Option<Vec<&'a str>>,
    default_syntax: Option<&'a str>,
    #[cfg_attr(feature = "serde", serde(default))]
    whitespace: WhitespaceHandling,
}

#[cfg_attr(feature = "serde", derive(Deserialize))]
struct RawSyntax<'a> {
    name: &'a str,
    block_start: Option<&'a str>,
    block_end: Option<&'a str>,
    expr_start: Option<&'a str>,
    expr_end: Option<&'a str>,
    comment_start: Option<&'a str>,
    comment_end: Option<&'a str>,
}

#[cfg_attr(feature = "serde", derive(Deserialize))]
struct RawEscaper<'a> {
    path: &'a str,
    extensions: Vec<&'a str>,
}

pub(crate) fn read_config_file(
    config_path: Option<&str>,
) -> std::result::Result<String, CompileError> {
    let root = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let filename = match config_path {
        Some(config_path) => root.join(config_path),
        None => root.join(CONFIG_FILE_NAME),
    };

    if filename.exists() {
        fs::read_to_string(&filename)
            .map_err(|_| format!("unable to read {:?}", filename.to_str().unwrap()).into())
    } else if config_path.is_some() {
        Err(format!("`{}` does not exist", root.display()).into())
    } else {
        Ok("".to_string())
    }
}

fn str_set<T>(vals: &[T]) -> HashSet<String>
where
    T: ToString,
{
    vals.iter().map(|s| s.to_string()).collect()
}

#[allow(clippy::match_wild_err_arm)]
pub(crate) fn get_template_source(tpl_path: &Path) -> std::result::Result<String, CompileError> {
    match fs::read_to_string(tpl_path) {
        Err(_) => Err(format!(
            "unable to open template file '{}'",
            tpl_path.to_str().unwrap()
        )
        .into()),
        Ok(mut source) => {
            if source.ends_with('\n') {
                let _ = source.pop();
            }
            Ok(source)
        }
    }
}

static CONFIG_FILE_NAME: &str = "askama.toml";
static DEFAULT_SYNTAX_NAME: &str = "default";
static DEFAULT_ESCAPERS: &[(&[&str], &str)] = &[
    (&["html", "htm", "svg", "xml"], "::askama::Html"),
    (&["md", "none", "txt", "yml", ""], "::askama::Text"),
    (&["j2", "jinja", "jinja2"], "::askama::Html"),
];

#[cfg(test)]
mod tests {
    use std::env;
    use std::path::{Path, PathBuf};

    use super::*;

    #[test]
    fn get_source() {
        let path = Config::new("", None)
            .and_then(|config| config.find_template("b.html", None))
            .unwrap();
        assert_eq!(get_template_source(&path).unwrap(), "bar");
    }

    #[test]
    fn test_default_config() {
        let mut root = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
        root.push("templates");
        let config = Config::new("", None).unwrap();
        assert_eq!(config.dirs, vec![root]);
    }

    #[cfg(feature = "config")]
    #[test]
    fn test_config_dirs() {
        let mut root = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
        root.push("tpl");
        let config = Config::new("[general]\ndirs = [\"tpl\"]", None).unwrap();
        assert_eq!(config.dirs, vec![root]);
    }

    fn assert_eq_rooted(actual: &Path, expected: &str) {
        let mut root = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
        root.push("templates");
        let mut inner = PathBuf::new();
        inner.push(expected);
        assert_eq!(actual.strip_prefix(root).unwrap(), inner);
    }

    #[test]
    fn find_absolute() {
        let config = Config::new("", None).unwrap();
        let root = config.find_template("a.html", None).unwrap();
        let path = config.find_template("sub/b.html", Some(&root)).unwrap();
        assert_eq_rooted(&path, "sub/b.html");
    }

    #[test]
    #[should_panic]
    fn find_relative_nonexistent() {
        let config = Config::new("", None).unwrap();
        let root = config.find_template("a.html", None).unwrap();
        config.find_template("c.html", Some(&root)).unwrap();
    }

    #[test]
    fn find_relative() {
        let config = Config::new("", None).unwrap();
        let root = config.find_template("sub/b.html", None).unwrap();
        let path = config.find_template("c.html", Some(&root)).unwrap();
        assert_eq_rooted(&path, "sub/c.html");
    }

    #[test]
    fn find_relative_sub() {
        let config = Config::new("", None).unwrap();
        let root = config.find_template("sub/b.html", None).unwrap();
        let path = config.find_template("sub1/d.html", Some(&root)).unwrap();
        assert_eq_rooted(&path, "sub/sub1/d.html");
    }

    #[cfg(feature = "config")]
    #[test]
    fn add_syntax() {
        let raw_config = r#"
        [general]
        default_syntax = "foo"

        [[syntax]]
        name = "foo"
        block_start = "{<"

        [[syntax]]
        name = "bar"
        expr_start = "{!"
        "#;

        let default_syntax = Syntax::default();
        let config = Config::new(raw_config, None).unwrap();
        assert_eq!(config.default_syntax, "foo");

        let foo = config.syntaxes.get("foo").unwrap();
        assert_eq!(foo.block_start, "{<");
        assert_eq!(foo.block_end, default_syntax.block_end);
        assert_eq!(foo.expr_start, default_syntax.expr_start);
        assert_eq!(foo.expr_end, default_syntax.expr_end);
        assert_eq!(foo.comment_start, default_syntax.comment_start);
        assert_eq!(foo.comment_end, default_syntax.comment_end);

        let bar = config.syntaxes.get("bar").unwrap();
        assert_eq!(bar.block_start, default_syntax.block_start);
        assert_eq!(bar.block_end, default_syntax.block_end);
        assert_eq!(bar.expr_start, "{!");
        assert_eq!(bar.expr_end, default_syntax.expr_end);
        assert_eq!(bar.comment_start, default_syntax.comment_start);
        assert_eq!(bar.comment_end, default_syntax.comment_end);
    }

    #[cfg(feature = "config")]
    #[test]
    fn add_syntax_two() {
        let raw_config = r#"
        syntax = [{ name = "foo", block_start = "{<" },
                  { name = "bar", expr_start = "{!" } ]

        [general]
        default_syntax = "foo"
        "#;

        let default_syntax = Syntax::default();
        let config = Config::new(raw_config, None).unwrap();
        assert_eq!(config.default_syntax, "foo");

        let foo = config.syntaxes.get("foo").unwrap();
        assert_eq!(foo.block_start, "{<");
        assert_eq!(foo.block_end, default_syntax.block_end);
        assert_eq!(foo.expr_start, default_syntax.expr_start);
        assert_eq!(foo.expr_end, default_syntax.expr_end);
        assert_eq!(foo.comment_start, default_syntax.comment_start);
        assert_eq!(foo.comment_end, default_syntax.comment_end);

        let bar = config.syntaxes.get("bar").unwrap();
        assert_eq!(bar.block_start, default_syntax.block_start);
        assert_eq!(bar.block_end, default_syntax.block_end);
        assert_eq!(bar.expr_start, "{!");
        assert_eq!(bar.expr_end, default_syntax.expr_end);
        assert_eq!(bar.comment_start, default_syntax.comment_start);
        assert_eq!(bar.comment_end, default_syntax.comment_end);
    }

    #[cfg(feature = "config")]
    #[test]
    fn longer_delimiters() {
        let raw_config = r#"
        [[syntax]]
        name = "emoji"
        block_start = "👉🙂👉"
        block_end = "👈🙃👈"
        expr_start = "🤜🤜"
        expr_end = "🤛🤛"
        comment_start = "👎_(ツ)_👎"
        comment_end = "👍:D👍"

        [general]
        default_syntax = "emoji"
        "#;

        let config = Config::new(raw_config, None).unwrap();
        assert_eq!(config.default_syntax, "emoji");

        let foo = config.syntaxes.get("emoji").unwrap();
        assert_eq!(foo.block_start, "👉🙂👉");
        assert_eq!(foo.block_end, "👈🙃👈");
        assert_eq!(foo.expr_start, "🤜🤜");
        assert_eq!(foo.expr_end, "🤛🤛");
        assert_eq!(foo.comment_start, "👎_(ツ)_👎");
        assert_eq!(foo.comment_end, "👍:D👍");
    }

    #[cfg(feature = "config")]
    #[test]
    fn illegal_delimiters() {
        let raw_config = r#"
        [[syntax]]
        name = "too_short"
        block_start = "<"
        "#;
        let config = Config::new(raw_config, None);
        assert_eq!(
            config.unwrap_err().msg,
            r#"delimiters must be at least two characters long: "<""#,
        );

        let raw_config = r#"
        [[syntax]]
        name = "contains_ws"
        block_start = " {{ "
        "#;
        let config = Config::new(raw_config, None);
        assert_eq!(
            config.unwrap_err().msg,
            r#"delimiters may not contain white spaces: " {{ ""#,
        );

        let raw_config = r#"
        [[syntax]]
        name = "is_prefix"
        block_start = "{{"
        expr_start = "{{$"
        comment_start = "{{#"
        "#;
        let config = Config::new(raw_config, None);
        assert_eq!(
            config.unwrap_err().msg,
            r#"a delimiter may not be the prefix of another delimiter: "{{" vs "{{$""#,
        );
    }

    #[cfg(feature = "toml")]
    #[should_panic]
    #[test]
    fn use_default_at_syntax_name() {
        let raw_config = r#"
        syntax = [{ name = "default" }]
        "#;

        let _config = Config::new(raw_config, None).unwrap();
    }

    #[cfg(feature = "toml")]
    #[should_panic]
    #[test]
    fn duplicated_syntax_name_on_list() {
        let raw_config = r#"
        syntax = [{ name = "foo", block_start = "~<" },
                  { name = "foo", block_start = "%%" } ]
        "#;

        let _config = Config::new(raw_config, None).unwrap();
    }

    #[cfg(feature = "toml")]
    #[should_panic]
    #[test]
    fn is_not_exist_default_syntax() {
        let raw_config = r#"
        [general]
        default_syntax = "foo"
        "#;

        let _config = Config::new(raw_config, None).unwrap();
    }

    #[cfg(feature = "config")]
    #[test]
    fn escape_modes() {
        let config = Config::new(
            r#"
            [[escaper]]
            path = "::askama::Js"
            extensions = ["js"]
        "#,
            None,
        )
        .unwrap();
        assert_eq!(
            config.escapers,
            vec![
                (str_set(&["js"]), "::askama::Js".into()),
                (
                    str_set(&["html", "htm", "svg", "xml"]),
                    "::askama::Html".into()
                ),
                (
                    str_set(&["md", "none", "txt", "yml", ""]),
                    "::askama::Text".into()
                ),
                (str_set(&["j2", "jinja", "jinja2"]), "::askama::Html".into()),
            ]
        );
    }

    #[cfg(feature = "config")]
    #[test]
    fn test_whitespace_parsing() {
        let config = Config::new(
            r#"
            [general]
            whitespace = "suppress"
            "#,
            None,
        )
        .unwrap();
        assert_eq!(config.whitespace, WhitespaceHandling::Suppress);

        let config = Config::new(r#""#, None).unwrap();
        assert_eq!(config.whitespace, WhitespaceHandling::Preserve);

        let config = Config::new(
            r#"
            [general]
            whitespace = "preserve"
            "#,
            None,
        )
        .unwrap();
        assert_eq!(config.whitespace, WhitespaceHandling::Preserve);

        let config = Config::new(
            r#"
            [general]
            whitespace = "minimize"
            "#,
            None,
        )
        .unwrap();
        assert_eq!(config.whitespace, WhitespaceHandling::Minimize);
    }

    #[cfg(feature = "toml")]
    #[test]
    fn test_whitespace_in_template() {
        // Checking that template arguments have precedence over general configuration.
        // So in here, in the template arguments, there is `whitespace = "minimize"` so
        // the `WhitespaceHandling` should be `Minimize` as well.
        let config = Config::new(
            r#"
            [general]
            whitespace = "suppress"
            "#,
            Some(&"minimize".to_owned()),
        )
        .unwrap();
        assert_eq!(config.whitespace, WhitespaceHandling::Minimize);

        let config = Config::new(r#""#, Some(&"minimize".to_owned())).unwrap();
        assert_eq!(config.whitespace, WhitespaceHandling::Minimize);
    }

    #[test]
    fn test_config_whitespace_error() {
        let config = Config::new(r#""#, Some("trim"));
        if let Err(err) = config {
            assert_eq!(err.msg, "invalid value for `whitespace`: \"trim\"");
        } else {
            panic!("Config::new should have return an error");
        }
    }
}
