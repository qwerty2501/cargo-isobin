#!/usr/bin/env bash
source $(dirname $0)/../prepare
cd $1
cargo clippy --all-features --tests -- -D clippy::all -D warnings --no-deps
cargo fmt -- --check
