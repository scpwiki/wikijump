#!/bin/bash
set -eu

run_sudo() {
	if [[ -n "${USE_SUDO:-}" ]]; then
		sudo "$@"
	else
		"$@"
	fi
}

cd install/local/dev
run_sudo docker-compose -p wikijump -f docker-compose.yaml -f docker-compose.dev.yaml up --build
