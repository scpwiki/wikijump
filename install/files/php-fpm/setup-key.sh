#!/bin/bash
set -eux

sed -i "s/APP_KEY=/APP_KEY=base64:$(head -c 32 /dev/random | base64)/" .env
