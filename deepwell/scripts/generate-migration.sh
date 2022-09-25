#!/bin/bash
set -eu

# Retrieve migration name
printf 'Enter migration name: '
read -r title

# Build filename
title="$(echo -n "$title" | tr '[:upper:]' '[:lower:]' | tr -c '[:alnum:]' '_')"
timestamp="$(date +%Y%m%d%H%M%S)"
migration_file="${timestamp}_${title}.sql"

# Create migration file
cd "${0%/*}/../migrations"
echo "Creating $migration_file..."
touch "$migration_file"
