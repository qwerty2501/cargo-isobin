#!/usr/bin/env bash
source $(dirname $0)/../prepare

scripts/test/test_rust_partial.sh .
