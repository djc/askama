#!/bin/bash
set -exuo pipefail
cd "${0%/*}"

export RUSTC_BOOTSTRAP=1
export RUSTFLAGS='-C passes=sancov-module -C llvm-args=-sanitizer-coverage-level=3 -C llvm-args=-sanitizer-coverage-inline-8bit-counters -Z sanitizer=address'
export HFUZZ_BUILD_ARGS='--features=honggfuzz'
exec cargo hfuzz run parse-random-code-honggfuzz
