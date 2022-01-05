#!/bin/bash

source $(dirname $0)/../../../prepare

if scripts/git/hooks/utils/is_remote_branch_wip.sh; then
	exit 0
fi

scripts/test/test_rust.sh
