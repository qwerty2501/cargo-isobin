#!/usr/bin/env bash
set -eu
cd $1
cargo test --all-features
