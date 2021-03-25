#!/bin/bash
set -eu

echo "Postgres version: $POSTGRES_VERSION"
echo "Postgres cluster: $POSTGRES_CLUSTER"
echo
echo "Running database in foreground..."

exec \
	pg_ctlcluster "$POSTGRES_VERSION" "$POSTGRES_CLUSTER" \
	start --foreground --stdlog
