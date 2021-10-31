#!/bin/bash
set -eux

# Start database
su postgres -c 'pg_ctl start'

# Prepare seed file
sed -i "s/FILEDOMAIN/${FILES_DOMAIN}/g" seed.sql

# Ingest database seed
su postgres -c 'psql < seed.sql'
