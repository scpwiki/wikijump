#!/bin/bash
set -eux

# Start database
service postgresql start

# Prepare seed file
gunzip seed.sql.gz
sed -i "s/FILEDOMAIN/${files_domain}/g" seed.sql

# Ingest database seed
su postgres -c 'psql < seed.sql'
