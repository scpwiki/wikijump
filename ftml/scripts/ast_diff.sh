#!/usr/bin/bash
set -eu

readonly newline=$'\n'

expected=''
actual=''

current='expected'

while read -r line; do
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
