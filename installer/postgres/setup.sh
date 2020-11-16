#!/bin/bash
set -eux

# Create postgres cluster
pg_createcluster -u postgres 12 main

# Start service
service postgresql start

# Set up database
su postgres -c 'psql < setup.sql'

# Ingest initial schema
su postgres -c 'psql < ingest.sql'
