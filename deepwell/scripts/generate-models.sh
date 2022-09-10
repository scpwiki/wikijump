#!/bin/bash
set -eu

# Switch to service root
cd "${0%/*}/.."

# Delete old models, if present
[[ -d src/models ]] && rm -r src/models

# Generate models
sea-orm-cli generate entity \
	-u postgres://wikijump:wikijump@localhost/wikijump \
	-o src/models \
	--with-serde both
