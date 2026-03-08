#!/bin/bash

# Replace paths to something generic
export RUSTFLAGS="--remap-path-prefix $HOME=/home/build"

cargo clippy
cargo build -j $(nproc) --release
