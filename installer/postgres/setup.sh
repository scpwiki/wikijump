#!/bin/bash
set -eux

service postgresql start
su postgres -c 'psql < ingest.sql'
