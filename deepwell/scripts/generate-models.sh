#!/bin/bash
set -eu

# Switch to service root
cd "${0%/*}/.."

# Generate models
sea-orm-cli generate entity \
	-u postgres://wikijump:wikijump@localhost/wikijump \
	-o src/models \
	--with-serde both
