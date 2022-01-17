#!/bin/bash
set -eu

function join_arr() {
	local IFS="$1"
	shift
	echo "$*"
}

# Only tables that are used are included
#
# Additionally, many Ozone tables have schemas
# incompatible with SeaORM.

tables=(
	file
	page
	page_category
	page_connection
	page_connection_missing
	page_edit_lock
	page_link
	page_metadata
	page_rate_vote
	page_revision
	settings
	site
	site_settings
	text
	user_block
	user_messages
	users
)

# Switch to service root
cd "${0%/*}/.."

# Delete old models
[[ -d src/models ]] && rm -r src/models

# Generate models
sea-orm-cli generate entity \
	-t "$(join_arr , "${tables[@]}")" \
	-u postgres://wikijump:wikijump@localhost/wikijump \
	-o src/models \
	--with-serde both
