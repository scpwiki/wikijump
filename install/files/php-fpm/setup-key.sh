#!/bin/bash
set -eux

key="$(head -c 32 /dev/random | base64 | sed -e 's/[\/&]/\\&/g')"
sed -i "s/APP_KEY=/APP_KEY=base64:/" .env
