#!/bin/bash

set -eu

dir=$1


procs=()
for entry in ${dir%/}/*
do
	script="$(basename $entry)"
	if [ "$(basename $0)" != "$script" ]; then
		echo "start run:$script"
		${dir%/}/$script &
		procs=("${procs[@]}" $!)
	fi
done


for proc in ${procs[@]};
do
	wait $proc
done
