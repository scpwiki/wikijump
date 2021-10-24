#!/bin/bash
set -eu

#
# Since creating a new migration requires 'php artisan',
# which loads all the PHP dependencies and such, which
# may not be perfectly in alignment outside of the
# local deploy container, and creating a new migration
# file doesn't actually *require* any of those things
# to be fixed, this script is a replacement that makes
# a new migration.
#

function cd_migrations_dir() {
	# The expression "${0%/*}" is the same as "$(dirname $0)"
	# but it doesn't require a subshell
	cd "${0%/*}/../database/migrations"
}

function check_name() {
	# Check if there are dashes in the name
	if [[ $1 == *-* ]]; then
		echo "Name may not contain dashes: $1" >&2
		exit 1
	fi

	# Check if there are underscores in the name
	if [[ $1 != *_* ]]; then
		echo "Name is not in snake_case: $1" >&2
		exit 1
	fi

	# Check that there aren't any uppercase letters
	if [[ $1 =~ [A-Z] ]]; then
		echo "Name may not contain uppercase letters: $1" >&2
		exit 1
	fi
}

function make_filename() {
	echo "$(date +%Y_%m_%d_%H%M%S)_$1.php"
}

function to_pascal_case() {
	# From https://unix.stackexchange.com/a/196241
	sed -r 's/(^|_)([a-z])/\U\2/g' <<< "$1"
}

function make_migration() {
	echo "+ $1"

	class_name="$(to_pascal_case "$1")"
	filename="$(make_filename "$1")"

	cat > "$filename" << EOF
<?php

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

class $class_name extends Migration
{
    /**
     * Run the migrations.
     *
     * @return void
     */
    public function up()
    {
        // TODO
    }

    /**
     * Reverse the migrations.
     *
     * @return void
     */
    public function down()
    {
        // TODO
    }
}
EOF
}

function main() {
	if [[ $# -eq 0 ]]; then
		echo "Usage: $0 <name-of-migration...>" >&2
		exit 1
	fi

	cd_migrations_dir

	for name in "$@"; do
		check_name "$name"
	done

	for name in "$@"; do
		make_migration "$name"
	done
}

main "$@"
