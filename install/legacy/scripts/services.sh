#!/bin/bash

# Variables

readonly delay=60
readonly services=(
	'postgresql'
	'memcached'
	'nginx'
	'%PHP%-fpm' # Dockerfile replaces %PHP% with php version
)

# Sanity check

if [[ %PHP% = %* ]]; then
	echo "Template variable %PHP% not replaced by Dockerfile!"
	exit 1
fi

# Functions

start_all() {
	echo "+ Starting services..."

	for name in "${services[@]}"; do
		service "$name" start
	done

	echo "------------------------"
	echo "+ All services online! +"
	echo "------------------------"
}

stop_all() {
	echo "+ Stopping services..."

	for name in "${services[@]}"; do
		service "$name" stop
	done

	echo "-------------------------"
	echo "+ All services stopped! +"
	echo "-------------------------"
}

poll_all() {
	echo "+ Checking services..."
	local success=true

	for name in "${services[@]}"; do
		if service "$name" status > /dev/null; then
			echo "  [✓] $name"
		else
			echo "  [✗] $name"
			success=false
		fi
	done

	if ! "$success"; then
		echo "+ Some services are down!"
		exit 1
	fi
}

poll_all_forever() {
	echo "+ Polling services... (every $delay seconds)"
	while true; do
		poll_all
		sleep "$delay"
	done
}
