use proc_macro::token_stream::IntoIter as ProcIter;
use proc_macro::{Delimiter, TokenTree};

use std::fmt;

#[derive(Clone, Default, Debug)]
pub(crate) struct Generics {
    inner: Vec<(String, Option<String>)>,
}

impl Generics {
    pub(crate) fn add_lifetime(&mut self, lifetime: &str, bounds: &str) {
        let lifetime = lifetime.trim();
        if lifetime.is_empty() {
            return;
        }
        let bounds = bounds.trim();
        if bounds.is_empty() {
            self.inner.insert(0, (lifetime.to_string(), None));
        } else {
            self.inner
                .insert(0, (lifetime.to_string(), Some(bounds.to_string())));
        }
    }

    pub(crate) fn add_generic(&mut self, bound_name: &str, bounds: &str) {
        let bound_name = bound_name.trim();
        let bounds = bounds.trim();

        if bound_name.is_empty() {
            // Nothing to do...
        } else if bounds.is_empty() {
            self.inner.push((bound_name.to_string(), None));
        } else {
            self.inner
                .push((bound_name.to_string(), Some(bounds.to_string())));
        }
    }

    pub(crate) fn display_with_bounds(&self) -> String {
        if self.inner.is_empty() {
            return String::new();
        }
        format!(
            "<{}>",
            self.inner
                .iter()
                .map(|(generic, bounds)| {
                    if let Some(bounds) = bounds {
                        format!("{generic}: {bounds}")
                    } else {
                        generic.clone()
                    }
                })
                .collect::<Vec<_>>()
                .join(",")
        )
    }
}

impl fmt::Display for Generics {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.inner.is_empty() {
            return Ok(());
        }
        write!(
            f,
            "<{}>",
            self.inner
                .iter()
                .filter(|(generic, _)| generic != "'_")
                .map(
                    |(generic, _)| if let Some(stripped) = generic.strip_prefix("const ") {
                        stripped
                    } else {
                        generic.as_str()
                    }
                )
                .collect::<Vec<_>>()
                .join(",")
        )
    }
}

#[derive(Clone, Default, Debug)]
pub(crate) struct WhereClause {
    inner: Option<String>,
}

#[cfg(feature = "with-mendes")]
impl WhereClause {
    pub(crate) fn add_predicate(&mut self, predicate: &str) {
        let predicate = predicate.trim();

        if predicate.is_empty() {
            return;
        }
        if let Some(inner) = self.inner.as_mut() {
            if !inner.ends_with(',') {
                inner.push(',');
            }
            inner.push_str(predicate);
        } else {
            self.inner = Some(predicate.to_string());
        }
    }
}

impl fmt::Display for WhereClause {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.inner {
            Some(ref inner) => write!(f, "where {inner}"),
            None => Ok(()),
        }
    }
}

fn skip_until_back_to_first_level(
    iter: &mut ProcIter,
    mut tokens_with_bounds: Option<&mut String>,
) {
    let mut level = 1;

    for token in iter {
        if let Some(tokens) = tokens_with_bounds.as_mut() {
            tokens.push_str(&token.to_string());
        }
        if let TokenTree::Punct(ref p) = token {
            match p.as_char() {
                '>' => {
                    level -= 1;
                    if level < 1 {
                        return;
                    }
                }
                '<' => level += 1,
                _ => {}
            }
        }
    }
}

fn skip_until_next_generic(iter: &mut ProcIter) -> bool {
    while let Some(token) = iter.next() {
        if let TokenTree::Punct(ref p) = token {
            match p.as_char() {
                ',' => return true,
                '>' => return false,
                '<' => skip_until_back_to_first_level(iter, None),
                _ => {}
            }
        }
    }
    false
}

fn only_bounds(iter: &mut ProcIter, tokens_with_bounds: &mut String) -> bool {
    while let Some(token) = iter.next() {
        if let TokenTree::Punct(ref p) = token {
            match p.as_char() {
                '>' => return false,
                '<' => skip_until_back_to_first_level(iter, Some(tokens_with_bounds)),
                ',' => return true,
                '=' => return skip_until_next_generic(iter),
                _ => {}
            }
        }
        tokens_with_bounds.push_str(&token.to_string());
    }
    false
}

/// Parses and retrieves type's ident, generics (if any) and where clauses (if any).
pub(crate) fn get_generics_and_type_name(
    iter: &mut ProcIter,
    ident: &mut Option<String>,
    generics: &mut Generics,
    where_clause: &mut WhereClause,
) {
    match iter.next() {
        Some(TokenTree::Ident(id)) => *ident = Some(id.to_string()),
        _ => return,
    }
    match iter.next() {
        Some(TokenTree::Punct(p)) if p.as_char() == '<' => {
            let mut generic = String::new();

            while let Some(token) = iter.next() {
                let should_continue = match token {
                    TokenTree::Punct(ref p) => {
                        match p.as_char() {
                            '>' => {
                                generics.add_generic(&generic, "");
                                break;
                            }
                            '=' => {
                                generics.add_generic(&generic, "");
                                generic.clear();
                                skip_until_next_generic(iter)
                            }
                            ':' => {
                                if let Some(new_token) = iter.next() {
                                    if matches!(new_token, TokenTree::Punct(ref p) if p.as_char() == ':')
                                    {
                                        // It's a path.
                                        generic.push_str("::");
                                    // It's generic bounds.
                                    } else {
                                        let mut bounds = new_token.to_string();
                                        let ret = only_bounds(iter, &mut bounds);
                                        generics.add_generic(&generic, &bounds);
                                        generic.clear();
                                        if !ret {
                                            break;
                                        }
                                    }
                                    continue;
                                } else {
                                    break;
                                }
                            }
                            ',' => {
                                generics.add_generic(&generic, "");
                                generic.clear();
                                continue;
                            }
                            _ => true,
                        }
                    }
                    _ => true,
                };
                generic.push_str(&token.to_string());
                if !should_continue {
                    break;
                }
                if matches!(token, TokenTree::Ident(_)) {
                    generic.push(' ');
                }
            }
        }
        _ => return,
    }
    match iter.next() {
        Some(TokenTree::Ident(id)) if id.to_string() == "where" => {}
        _ => return,
    }
    let mut tokens = String::new();
    let mut bracket_group_level = 0;
    while let Some(token) = iter.next() {
        match token {
            TokenTree::Punct(ref p) => {
                // As long as we're inside this, we need to ignore bracket groups to handle
                // cases like:
                //
                // ```
                // trait Bar<const X: usize>{}
                // struct Foo<F> where F: Bar<{12}> { inner: F }
                // ```
                match p.as_char() {
                    '<' => bracket_group_level += 1,
                    '>' => bracket_group_level -= 1,
                    ':' if bracket_group_level == 0 => {
                        if let Some(token) = iter.next() {
                            if matches!(token, TokenTree::Punct(ref p) if p.as_char() == ':') {
                                // it's a path
                                tokens.push_str("::");
                            } else {
                                tokens.push_str(": ");
                                tokens.push_str(&token.to_string());
                            }
                            continue;
                        }
                    }
                    _ => {}
                }
            }
            TokenTree::Group(ref g)
                if bracket_group_level == 0 && g.delimiter() != Delimiter::None =>
            {
                break
            }
            _ => {}
        }
        tokens.push_str(&token.to_string());
    }
    where_clause.inner = Some(tokens);
}
