#!/bin/bash
set -eux

# Initialize database
su postgres -c "initdb -D ${PGDATA} -A trust --locale=en_US.UTF-8"
install -D -m644 -o postgres -g postgres /src/pg_hba.conf "${PGDATA}/pg_hba.conf"

# Start database
su postgres -c "pg_ctl -D ${PGDATA} start"

# Prepare seed file
sed -i "s/FILEDOMAIN/${FILES_DOMAIN}/g" seed.sql

# Ingest database seed
su postgres -c 'psql < seed.sql'
