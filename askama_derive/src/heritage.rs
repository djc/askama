use std::collections::HashMap;
use std::path::Path;
use std::rc::Rc;

use crate::config::Config;
use crate::CompileError;
use parser::node::{BlockDef, Macro, Match};
use parser::Node;

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

type BlockAncestry<'a> = HashMap<&'a str, Vec<(&'a Context<'a>, &'a BlockDef)>>;

#[derive(Default, Clone)]
pub(crate) struct Context<'a> {
    pub(crate) nodes: &'a [Node],
    pub(crate) extends: Option<Rc<Path>>,
    pub(crate) blocks: HashMap<&'a str, &'a BlockDef>,
    pub(crate) macros: HashMap<&'a str, &'a Macro>,
    pub(crate) imports: HashMap<&'a str, Rc<Path>>,
}

impl Context<'_> {
    pub(crate) fn new<'n>(
        config: &Config<'_>,
        path: &Path,
        nodes: &'n [Node],
    ) -> Result<Context<'n>, CompileError> {
        let mut extends = None;
        let mut blocks = HashMap::new();
        let mut macros = HashMap::new();
        let mut imports = HashMap::new();
        let mut nested = vec![nodes];
        let mut top = true;

        while let Some(nodes) = nested.pop() {
            for n in nodes {
                match n {
                    Node::Extends(e) if top => match extends {
                        Some(_) => return Err("multiple extend blocks found".into()),
                        None => {
                            extends = Some(config.find_template(e.path.as_str(), Some(path))?);
                        }
                    },
                    Node::Macro(m) if top => {
                        macros.insert(m.name.as_str(), m);
                    }
                    Node::Import(import) if top => {
                        let path = config.find_template(import.path.as_str(), Some(path))?;
                        imports.insert(import.scope.as_str(), path);
                    }
                    Node::Extends(_) | Node::Macro(_) | Node::Import(_) if !top => {
                        return Err(
                            "extends, macro or import blocks not allowed below top level".into(),
                        );
                    }
                    Node::BlockDef(b) => {
                        blocks.insert(b.name.as_str(), b);
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
                    Node::Match(Match { arms, .. }) => {
                        for arm in arms {
                            nested.push(&arm.nodes);
                        }
                    }
                    _ => {}
                }
            }
            top = false;
        }

        Ok(Context {
            nodes,
            extends,
            blocks,
            macros,
            imports,
        })
    }
}
