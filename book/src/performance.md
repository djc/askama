# Performance

## Slow Debug Recompilations

If you experience slow compile times when iterating with lots of templates,
you can compile Askama's derive macros with a higher optimization level.
This can speed up recompilation times dramatically.

Add the following to `Cargo.toml` or `.cargo/config.toml`:
```toml
[profile.dev.package.askama_derive]
opt-level = 3
```

This may affect clean compile times in debug mode, but incremental compiles
will be faster.
