#!/bin/bash

set -eu

echo $(git diff --name-only --cached --diff-filter=d -- $@)
