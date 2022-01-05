#!/bin/bash

set -eu

branch_name="${GIT_HOOKS_REMOTE_REF##refs/heads/}"
if [[ "$branch_name" == "wip/"* ]]; then
	exit 0
fi

exit 1

