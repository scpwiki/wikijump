#!/bin/sh
set -e

rm -f /tmp/deepwell.json /tmp/Caddyfile

# Send DEEPWELL request
curl -f http://deepwell:2747/jsonrpc \
	-X POST \
	--json @/etc/caddy-request.json \
		> /tmp/deepwell.json

# Determine if it's an error
error="$(jq .error /tmp/deepwell.json)"
if [ "$error" != null ]; then
	cat /tmp/deepwell.json
	exit 1
fi

# Call was a success, extract the Caddyfile
jq -r .result /tmp/deepwell.json > /tmp/Caddyfile
rm /tmp/deepwell.json
