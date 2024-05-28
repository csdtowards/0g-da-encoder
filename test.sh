#!/bin/bash

# 当出现错误时，停止执行后续命令
set -e

export RUSTFLAGS="-D warnings" 

if [[ ! -f crates/ppot2ark/data/challenge_12 ]]; then
    cd crates/ppot2ark
    ./gen_test_ppot.sh 12
    cd ../..
fi

cargo run -r -p amt --features parallel --bin build_params -- 11 6 0
cargo run -r -p amt --features parallel --bin build_params -- 11 6 1
cargo run -r -p amt --features parallel --bin build_params -- 11 6 2

cargo check --all
cargo check --all --features parallel
cargo check --all --features cuda

cargo check --all --tests --benches
cargo check --all --tests --benches --features parallel
cargo check --all --tests --benches --features cuda

cargo test -r --all --features parallel