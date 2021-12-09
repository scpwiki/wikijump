#!/bin/bash
set -eu

# Only tables that are used are included
#
# Additionally, many Ozone tables have schemas
# incompatible with SeaORM.

tables=(
	page
	page_connection
	page_connection_missing
	page_contents
	page_edit_lock
	page_link
	page_metadata
	page_rate_vote
	page_revision
	users
)

# Switch to service root
cd "${0%/*}/.."

# Delete old models
rm -r src/models

# Generate models
sea-orm-cli generate entity \
	-t "${tables%,}" \
	-u postgres://wikijump:wikijump@localhost/wikijump \
	-o src/models
