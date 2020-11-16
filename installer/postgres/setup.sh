#!/bin/bash
set -eux

# Create postgres cluster
pg_createcluster -u postgres 12 main

# Start service
service postgresql start

# Ingest database seed
su postgres -c 'psql < ingest.sql'
