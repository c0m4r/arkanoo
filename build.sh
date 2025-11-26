#!/bin/bash

cargo clippy
cargo build -j $(nproc) --release
