#!/usr/bin/bash
set -eu

#
# Help with comparing outputs form AST tests.
#

readonly newline=$'\n'

expected=''
actual=''

current='expected'

while IFS= read -r line; do
	if [[ $line == Actual:* ]]; then
		current='actual'
	fi

	case "$current" in
		expected)
			expected+="${line}${newline}"
			;;
		actual)
			actual+="${line}${newline}"
			;;
	esac
done

diff --color <(echo "$expected") <(echo "$actual")
