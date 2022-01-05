#!/bin/bash

source $(dirname $0)/../../../prepare

file_paths=$(scripts/git/hooks/utils/staged_exists_files.sh *.rs)
if [ -n "$file_paths"  ]; then
	cargo fmt -- $file_paths
	git add $file_paths
fi
