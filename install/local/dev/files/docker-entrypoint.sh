#!/bin/bash
set -eu

#
# Adapted from https://gist.github.com/cxcorp/f4eec6575a815edcb63731da93362504
#

# Set variables

readonly data_dir="${DATA_DIR:-/data}"
readonly api_address='localhost:9000'
readonly console_address='localhost:9001'

readonly mc_user="${MINIO_ROOT_USER:-minioadmin}"
readonly mc_password="${MINIO_ROOT_PASSWORD:-minioadmin}"
readonly mc_region="${MINIO_REGION_NAME:-us-east-1}"

# Helper functions

function wait_for_server() {
	until curl -fs "http://$api_address"; do
		sleep 1
		echo "Waiting..."
	done
}

function create_bucket() {
	mc alias -q set local "http://$api_address" "$mc_user" "$mc_password" > /dev/null

	local address="local/$1"
	if mc stat -q "$address" > /dev/null; then
		echo "Bucket $1 already exists"
		return
	fi

	echo "Creating bucket $1"
	mc mb --region "$mc_region" -p "$address"
}

function create_initial_buckets() {
	echo "Creating initial buckets: ${INITIAL_BUCKETS:-}"
	IFS=' ' read -r -a buckets <<< "${INITIAL_BUCKETS:-}"

	# Start server in background
	minio server "$data_dir" \
		--quiet \
		--address "$api_address" \
		--console-address "$console_address" \
		&
	local pid="$!"

	echo "Server started, waiting for console to come up"
	wait_for_server

	echo "Server is ready. Creating missing buckets."
	for bucket in "${buckets[@]}"; do
		create_bucket "$bucket"
	done

	echo "Bucket setup complete, terminating temporary server"
	kill "$pid" && wait "$pid"
}

# Main

create_initial_buckets

exec minio server "$data_dir" \
	--quiet \
	--address "$api_address" \
	--console-address "$console_address"
