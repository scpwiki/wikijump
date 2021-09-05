#!/usr/bin/bash
set -eu

#
# Create stubs for AST tests.
#

for name in "$@"; do
	echo "Creating $name..."

	cp misc/ast-test-template.json "test/$name.json"
	echo > "test/$name.txt"
	echo > "test/$name.html"
done
