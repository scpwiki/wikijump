#!/bin/bash
set -eux

# Start database
service postgresql start

# Prepare seed file
sed -i "s/FILEDOMAIN/${FILES_DOMAIN}/g" seed.sql

# Ingest database seed
su postgres -c 'psql < seed.sql'

# Install run script
install -D -m755 /src/run.sh /app/run.sh
