use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::parser::{Expr, Macro, Node};
use crate::Config;

pub struct Heritage<'a> {
    pub root: &'a Context<'a>,
    pub blocks: BlockAncestry<'a>,
}

impl<'a> Heritage<'a> {
    pub fn new<'n>(
        mut ctx: &'n Context<'n>,
        contexts: &'n HashMap<&'n PathBuf, Context<'n>>,
    ) -> Heritage<'n> {
        let mut blocks: BlockAncestry<'n> = ctx
            .blocks
            .iter()
            .map(|(name, def)| (*name, vec![(ctx, *def)]))
            .collect();

        while let Some(ref path) = ctx.extends {
            ctx = &contexts[&path];
            for (name, def) in &ctx.blocks {
                blocks.entry(name).or_insert_with(Vec::new).push((ctx, def));
            }
        }

        Heritage { root: ctx, blocks }
    }
}

type BlockAncestry<'a> = HashMap<&'a str, Vec<(&'a Context<'a>, &'a Node<'a>)>>;

pub struct Context<'a> {
    pub nodes: &'a [Node<'a>],
    pub extends: Option<PathBuf>,
    pub blocks: HashMap<&'a str, &'a Node<'a>>,
    pub macros: HashMap<&'a str, &'a Macro<'a>>,
    pub imports: HashMap<&'a str, PathBuf>,
}

impl<'a> Context<'a> {
    pub fn new<'n>(config: &Config, path: &Path, nodes: &'n [Node<'n>]) -> Context<'n> {
        let mut extends = None;
        let mut blocks = Vec::new();
        let mut macros = HashMap::new();
        let mut imports = HashMap::new();

        for n in nodes {
            match n {
                Node::Extends(Expr::StrLit(extends_path)) => match extends {
                    Some(_) => panic!("multiple extend blocks found"),
                    None => {
                        extends = Some(config.find_template(extends_path, Some(path)));
                    }
                },
                def @ Node::BlockDef(_, _, _, _) => {
                    blocks.push(def);
                }
                Node::Macro(name, m) => {
                    macros.insert(*name, m);
                }
                Node::Import(_, import_path, scope) => {
                    let path = config.find_template(import_path, Some(path));
                    imports.insert(*scope, path);
                }
                _ => {}
            }
        }

        let mut check_nested = 0;
        let mut nested_blocks = Vec::new();
        while check_nested < blocks.len() {
            if let Node::BlockDef(_, _, ref nodes, _) = blocks[check_nested] {
                for n in nodes {
                    if let def @ Node::BlockDef(_, _, _, _) = n {
                        nested_blocks.push(def);
                    }
                }
            } else {
                panic!("non block found in list of blocks");
            }
            blocks.append(&mut nested_blocks);
            check_nested += 1;
        }

        let blocks: HashMap<_, _> = blocks
            .iter()
            .map(|def| {
                if let Node::BlockDef(_, name, _, _) = def {
                    (*name, *def)
                } else {
                    unreachable!()
                }
            })
            .collect();

        Context {
            nodes,
            extends,
            blocks,
            macros,
            imports,
        }
    }
}
