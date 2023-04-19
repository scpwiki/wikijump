#!/bin/bash
set -eu

# Switch to service root
cd "${0%/*}/.."

# Delete old models, if present
[[ -d src/models ]] && rm -r src/models

# Generate models
sea-orm-cli generate entity \
	--verbose \
	-database-url postgres://wikijump:wikijump@localhost/wikijump \
	-output-dir src/models \
	--date-time-crate time \
	--with-serde both
