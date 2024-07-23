#!/usr/bin/env bash

# This script is intended to check if there are no "DNC" ("Do Not Commit") comments in the codebase.

SCRIPT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &>/dev/null && pwd)
ROOT_DIR=$(realpath $SCRIPT_DIR/..)

files_to_check=$(git ls-files)

found_dnc=false

for file in $files_to_check; do
	if [[ $file == *"dnc.sh" || $file == *".github/workflows/"* ]]; then
		continue
	fi

	file_path="$ROOT_DIR/$file"
	bad_command=$(rg -C3 "(DNC|dnc)(\(.*\))*:?\s?" $file_path)
	if [ -n "$bad_command" ]; then
		found_dnc=true

		echo "Found DNC comment!"
		echo

		echo "FILE PATH:"
		echo $file_path

		echo "MATCH:"
		echo "$bad_command"
	fi
done

if [ "$found_dnc" = true ]; then
	exit 1
fi