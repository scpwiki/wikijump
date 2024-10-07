#!/bin/bash
set -eu

#
# Helper script to upload files to a local S3 store for testing upload flows.
#

if [[ $# -ne 2 ]]; then
	echo >&2 "Usage: $0 <path-to-file> <s3-presign-url>"
	exit 1
fi

# Allow either order of arguments, for convenience.
# If it starts with HTTP or HTTPS, we assume it's the presign URL.
if [[ $1 = http:* || $1 = https:* ]]; then
	path="$2"
	url="$1"
else
	path="$1"
	url="$2"
fi

exec \
	curl \
		--connect-to 'files:9000:localhost:9000' \
		--upload-file "$path" \
		"$url"
