#!/bin/bash
set -eux

service postgresql start
su postgres -c 'psql < postgres-ingest.sql'
