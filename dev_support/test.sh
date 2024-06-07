#!/bin/bash

./dev_support/check_cuda.sh
CUDA_TEST_EXITCODE=$?
if [[ $CUDA_TEST_EXITCODE -ne 0 ]]; then
    echo "\033[1;33mCUDA Environment check fails, skip CUDA related tests\033[0m"
fi

set -e

export RUSTFLAGS="-D warnings" 

if [[ ! -f crates/ppot2ark/data/challenge_12 ]]; then
    cd crates/ppot2ark
    ./gen_test_ppot.sh 12
    cd ../..
fi

# cargo run -r -p amt --features parallel --bin build_params -- 11 6 0
# cargo run -r -p amt --features parallel --bin build_params -- 11 6 1
# cargo run -r -p amt --features parallel --bin build_params -- 11 6 2

cargo check --all
cargo check --all --features parallel

cargo check --all --tests --benches
cargo check --all --tests --benches --features parallel


if [[ $CUDA_TEST_EXITCODE -eq 0 ]]; then
    cargo check --all --features cuda
    cargo check --all --tests --benches --features cuda
fi

rm -rf "./crates/amt/pp/*-11.bin"
rm -rf "./crates/amt/pp/*-08.bin"

cargo test -r --all --features parallel

rm -rf "./crates/amt/pp/*-11.bin"
rm -rf "./crates/amt/pp/*-08.bin"

if [[ $CUDA_TEST_EXITCODE -eq 0 ]]; then
    cargo test -r -p amt --features amt/parallel,amt/cuda-bn254
    cargo test -r -p amt --features amt/parallel,amt/cuda-bls12-381
fi