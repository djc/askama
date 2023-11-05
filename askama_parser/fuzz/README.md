# Fuzzing

Install `cargo-fuzz`:

```sh
cargo install -f cargo-fuzz
```

Run any available target where `$target` is the name of the target.

```sh
cargo fuzz list # get list of targets
cargo +nightly fuzz run $target
```