#!/bin/bash

echoStep() {
    echo -e "\n\033[1;34m────────────────────────────────────────────────────────"
    echo -e "\033[1;34m$1."
    echo -e "\033[1;34m────────────────────────────────────────────────────────\033[0m"
}

./dev_support/check_cuda.sh
CUDA_TEST_EXITCODE=$?
if [[ $CUDA_TEST_EXITCODE -ne 80 ]]; then
    echo ""
    echo -e "    \033[1;33mCUDA Environment check fails, some CUDA related tests will be delete\033[0m"
    echo ""
fi

set -e

echoStep "Check fmt"
./cargo_fmt.sh -- --check

echoStep "Build ppot2ark test params"
export RUSTFLAGS="-A warnings" 
if [[ ! -f crates/ppot2ark/data/challenge_12 ]]; then
    cd crates/ppot2ark
    ./gen_test_ppot.sh 12
    cd ../..
fi

export RUSTFLAGS="-D warnings" 

echoStep "Check all"
cargo check --all
echoStep "Check all (parallel)"
cargo check --all --features parallel

echoStep "Check all tests"
cargo check --all --tests --benches
echoStep "Check all tests (parallel)"
cargo check --all --tests --benches --features parallel


if [[ $CUDA_TEST_EXITCODE -ne 80 ]]; then
    echoStep "Check all (cuda)"
    cargo check --all --features cuda
    echoStep "Check all tests (cuda)"
    cargo check --all --tests --benches --features cuda
fi

echoStep "Check clippy"
cargo clippy
echoStep "Check clippy (parallel)"
cargo clippy --features parallel
if [[ $CUDA_TEST_EXITCODE -ne 80 ]]; then
    echoStep "Check clippy (cuda)"
    cargo clippy --features cuda
fi 


rm -rf "./crates/amt/pp/*-11.bin"
rm -rf "./crates/amt/pp/*-08.bin"

echoStep "Test (parallel)"
cargo test -r --all --features parallel

rm -rf "./crates/amt/pp/*-11.bin"
rm -rf "./crates/amt/pp/*-08.bin"

if [[ $CUDA_TEST_EXITCODE -eq 0 ]]; then
    echoStep "Test (cuda-bn254)"
    cargo test -r -p amt --features amt/parallel,amt/cuda-bn254
    echoStep "Test (cuda-bls12-381)"
    cargo test -r -p amt --features amt/parallel,amt/cuda-bls12-381
fi