#!/bin/bash
set -eux

# Create postgres cluster
pg_createcluster -u postgres 12 main

# Start service
service postgresql start

# Create postgres database
createdb \
	--echo
	--encoding='UTF-8' \
	--owner=wikijump \
	wikijump \
	'Wikijump system database'

# Ingest database seed
su postgres -c 'psql < ingest.sql'
