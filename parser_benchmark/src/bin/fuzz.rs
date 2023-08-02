use std::env::temp_dir;
use std::fs::OpenOptions;
use std::io::Write;
use std::mem::forget;
use std::process::exit;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::{AcqRel, Acquire};
use std::sync::Mutex;
use std::thread::{scope, Builder};
use std::time::{Duration, Instant};

use gumdrop::Options;
use indicatif::{ProgressBar, ProgressStyle};
use parser::{Ast, ParseError, Syntax};
use parser_benchmark::{one_node_with_rng, Seed};

#[cfg(not(test))]
const DEFAULT_DURATION_IN_SECS: u64 = 120;
#[cfg(test)]
const DEFAULT_DURATION_IN_SECS: u64 = 10;

#[derive(Debug, Options)]
struct Args {
    #[options(help = "seed to initialize the RNG (default: random)")]
    seed: Option<u64>,
    #[options(help = "seconds to run the fuzzer (default: 120)")]
    seconds: Option<u64>,
    #[options(help = "use this many threads (default: 1 per CPU thread)")]
    threads: Option<usize>,
    #[options(help = "ignore any parser errors")]
    ignore: bool,
    #[options(help = "show progress bar")]
    progress: bool,
    #[options(help = "print help message")]
    help: bool,
}

#[cold]
fn fail(failed: &Mutex<()>, err: ParseError, source: &str) {
    forget(failed.lock().unwrap());

    let path = temp_dir().join("failed-askama-fuzz.jinja2");
    OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&path)
        .expect("should be able to write to temp dir")
        .write_all(source.as_bytes())
        .expect("should be able to write to temp dir");
    eprintln!(
        "\nCould not parse source:\n{}\n\nDumped source into: {:?}",
        err, path,
    );

    exit(1);
}

fn main() {
    let opts = Args::parse_args_default_or_exit();
    let thread_count = opts.threads.unwrap_or_else(num_cpus::get).max(1);
    let limit = Duration::from_secs(opts.seconds.unwrap_or(DEFAULT_DURATION_IN_SECS));
    let progress_bar = opts.progress.then(|| {
        const STYLE: &str = "{spinner} [{elapsed_precise}] [{wide_bar}]";
        let style = ProgressStyle::with_template(STYLE).unwrap();
        ProgressBar::new(limit.as_secs()).with_style(style)
    });

    let start = Instant::now();
    let count_success = AtomicUsize::new(0);
    let count_errors = AtomicUsize::new(0);
    let syntax = Syntax::default();
    let failed = Mutex::new(());

    scope(|s| {
        let mut threads = Vec::with_capacity(thread_count);
        for i in 0..thread_count {
            let count_success = &count_success;
            let count_errors = &count_errors;
            let syntax = &syntax;
            let failed = &failed;
            let progress_bar = match i {
                0 => progress_bar.clone(),
                _ => None,
            };

            let thread = Builder::new()
                .name(format!("#{i}"))
                .spawn_scoped(s, move || {
                    let seed = match opts.seed {
                        Some(seed) => Seed::Partial(seed + i as u64),
                        None => Seed::Random,
                    };
                    let mut rng = seed.into_rng();
                    let mut src = String::with_capacity(8192);
                    loop {
                        let elapsed = start.elapsed();
                        if elapsed >= limit {
                            break true;
                        }
                        if let Some(progress_bar) = &progress_bar {
                            progress_bar.set_position(elapsed.as_secs());
                        }

                        src.clear();
                        one_node_with_rng(&mut rng, &mut src);
                        if let Err(err) = Ast::from_str(&src, syntax) {
                            count_errors.fetch_add(1, AcqRel);
                            if !opts.ignore {
                                fail(failed, err, &src);
                            }
                        } else {
                            count_success.fetch_add(1, AcqRel);
                        }
                    }
                })
                .expect("should be able to start threads");
            threads.push(thread);
        }
        while let Some(thread) = threads.pop() {
            thread.join().expect("should be able to join thread");
        }
    });

    if let Some(progress_bar) = progress_bar {
        progress_bar.finish();
    }
    let count_success = count_success.load(Acquire);
    let count_errors = count_errors.load(Acquire);
    println!("Successful:     {count_success}");
    println!("Parsing errors: {count_errors}");
    if count_errors > 0 {
        exit(1);
    }
}
