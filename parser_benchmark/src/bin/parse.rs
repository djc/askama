use std::fs::OpenOptions;
use std::io::{stdin, stdout, Read, Write};
use std::path::PathBuf;

use gumdrop::Options;
use parser::{Ast, ParseError, Syntax};

#[derive(Debug, Options)]
struct Args {
    #[options(help = "file to read from (default: STDIN)")]
    input: Option<PathBuf>,
    #[options(help = "file to write to (default: STDOUT)")]
    output: Option<PathBuf>,
    #[options(help = "Syntax::block_start")]
    block_start: Option<String>,
    #[options(help = "Syntax::block_end")]
    block_end: Option<String>,
    #[options(help = "Syntax::expr_start")]
    expr_start: Option<String>,
    #[options(help = "Syntax::expr_end")]
    expr_end: Option<String>,
    #[options(help = "Syntax::comment_start")]
    comment_start: Option<String>,
    #[options(help = "Syntax:comment_end")]
    comment_end: Option<String>,
    #[options(help = "print help message")]
    help: bool,
}

#[derive(pretty_error_debug::Debug, thiserror::Error)]
enum Error {
    #[error("could not open input")]
    InOpen(#[source] std::io::Error),
    #[error("could not read input")]
    InRead(#[source] std::io::Error),
    #[error("could not open output")]
    OutOpen(#[source] std::io::Error),
    #[error("could not write output")]
    OutWrite(#[source] std::io::Error),
    #[error("could not parse source")]
    Parse(#[source] ParseError),
}

fn main() -> Result<(), Error> {
    let opts = Args::parse_args_default_or_exit();

    let source = {
        let mut file;
        let mut stdin_guard;
        let input: &mut dyn Read = match opts.input {
            Some(path) => {
                file = OpenOptions::new()
                    .read(true)
                    .open(path)
                    .map_err(Error::InOpen)?;
                &mut file
            }
            None => {
                stdin_guard = stdin().lock();
                &mut stdin_guard
            }
        };
        let mut source = String::new();
        input.read_to_string(&mut source).map_err(Error::InRead)?;
        source
    };

    let default_syntax = Syntax::default();
    let syntax = Syntax {
        block_start: opts
            .block_start
            .as_deref()
            .unwrap_or(default_syntax.block_start),
        block_end: opts
            .block_end
            .as_deref()
            .unwrap_or(default_syntax.block_end),
        expr_start: opts
            .expr_start
            .as_deref()
            .unwrap_or(default_syntax.expr_start),
        expr_end: opts.expr_end.as_deref().unwrap_or(default_syntax.expr_end),
        comment_start: opts
            .comment_start
            .as_deref()
            .unwrap_or(default_syntax.comment_start),
        comment_end: opts
            .comment_end
            .as_deref()
            .unwrap_or(default_syntax.comment_end),
    };

    let ast = Ast::from_str(&source, &syntax).map_err(Error::Parse)?;
    let ast: String = format!("{ast:?}");

    {
        let mut file;
        let mut stdout_guard;
        let output: &mut dyn Write = match opts.output {
            Some(path) => {
                file = OpenOptions::new()
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .open(path)
                    .map_err(Error::OutOpen)?;
                &mut file
            }
            None => {
                stdout_guard = stdout().lock();
                &mut stdout_guard
            }
        };
        output.write_all(ast.as_bytes()).map_err(Error::OutWrite)?;
    }

    Ok(())
}
