use std::collections::HashMap;
use std::path::Path;
use std::rc::Rc;

use crate::config::Config;
use crate::{CompileError, FileInfo};
use parser::node::{BlockDef, Macro};
use parser::{Node, Parsed, WithSpan};

pub(crate) struct Heritage<'a> {
    pub(crate) root: &'a Context<'a>,
    pub(crate) blocks: BlockAncestry<'a>,
}

impl Heritage<'_> {
    pub(crate) fn new<'n>(
        mut ctx: &'n Context<'n>,
        contexts: &'n HashMap<&'n Rc<Path>, Context<'n>>,
    ) -> Heritage<'n> {
        let mut blocks: BlockAncestry<'n> = ctx
            .blocks
            .iter()
            .map(|(name, def)| (*name, vec![(ctx, *def)]))
            .collect();

        while let Some(path) = &ctx.extends {
            ctx = &contexts[path];
            for (name, def) in &ctx.blocks {
                blocks.entry(name).or_default().push((ctx, def));
            }
        }

        Heritage { root: ctx, blocks }
    }
}

type BlockAncestry<'a> = HashMap<&'a str, Vec<(&'a Context<'a>, &'a BlockDef<'a>)>>;

#[derive(Clone)]
pub(crate) struct Context<'a> {
    pub(crate) nodes: &'a [Node<'a>],
    pub(crate) extends: Option<Rc<Path>>,
    pub(crate) blocks: HashMap<&'a str, &'a BlockDef<'a>>,
    pub(crate) macros: HashMap<&'a str, &'a Macro<'a>>,
    pub(crate) imports: HashMap<&'a str, Rc<Path>>,
    path: Option<&'a Path>,
    parsed: &'a Parsed,
}

impl Context<'_> {
    pub(crate) fn empty(parsed: &Parsed) -> Context<'_> {
        Context {
            nodes: &[],
            extends: None,
            blocks: HashMap::new(),
            macros: HashMap::new(),
            imports: HashMap::new(),
            path: None,
            parsed,
        }
    }

    pub(crate) fn new<'n>(
        config: &Config<'_>,
        path: &'n Path,
        parsed: &'n Parsed,
    ) -> Result<Context<'n>, CompileError> {
        let mut extends = None;
        let mut blocks = HashMap::new();
        let mut macros = HashMap::new();
        let mut imports = HashMap::new();
        let mut nested = vec![parsed.nodes()];
        let mut top = true;

        while let Some(nodes) = nested.pop() {
            for n in nodes {
                match n {
                    Node::Extends(e) if top => match extends {
                        Some(_) => return Err("multiple extend blocks found".into()),
                        None => {
                            extends = Some(config.find_template(e.path, Some(path))?);
                        }
                    },
                    Node::Macro(m) if top => {
                        macros.insert(m.name, &**m);
                    }
                    Node::Import(import) if top => {
                        let path = config.find_template(import.path, Some(path))?;
                        imports.insert(import.scope, path);
                    }
                    Node::Extends(_) | Node::Macro(_) | Node::Import(_) if !top => {
                        return Err(
                            "extends, macro or import blocks not allowed below top level".into(),
                        );
                    }
                    Node::BlockDef(b) => {
                        blocks.insert(b.name, &**b);
                        nested.push(&b.nodes);
                    }
                    Node::If(i) => {
                        for cond in &i.branches {
                            nested.push(&cond.nodes);
                        }
                    }
                    Node::Loop(l) => {
                        nested.push(&l.body);
                        nested.push(&l.else_nodes);
                    }
                    Node::Match(m) => {
                        for arm in &m.arms {
                            nested.push(&arm.nodes);
                        }
                    }
                    _ => {}
                }
            }
            top = false;
        }

        Ok(Context {
            nodes: parsed.nodes(),
            extends,
            blocks,
            macros,
            imports,
            parsed,
            path: Some(path),
        })
    }

    pub(crate) fn generate_error<T>(&self, msg: &str, node: &WithSpan<'_, T>) -> CompileError {
        match self.path {
            Some(path) => CompileError::new(
                msg,
                Some(FileInfo::new(
                    path,
                    Some(self.parsed.source()),
                    Some(node.span()),
                )),
            ),
            None => CompileError::new(msg, None),
        }
    }
}
