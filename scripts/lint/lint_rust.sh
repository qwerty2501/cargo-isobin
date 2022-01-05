#!/usr/bin/env bash
source $(dirname $0)/../prepare

find ./ -name '*.rs' -exec touch {} \;
scripts/lint/lint_rust_partial.sh .
