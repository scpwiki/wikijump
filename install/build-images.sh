#!/bin/bash
set -eu

readonly tag='local'
readonly images=(
	'ftml'
	'locales'
	'client'

	# Depends on the previous ones, must be last
	'php-fpm'
)

function run_docker() {
	if "${USE_SUDO:-false}"; then
		sudo docker "$@"
	else
		docker "$@"
	fi
}

function build_image() {
	env="$1"
	name="$2"
	image="scpwiki/$name:$tag"

	echo "Building image $image..."
	run_docker build -f "$docker_dir/$name/Dockerfile" -t "$image" .
}

if [[ $# -eq 0 ]]; then
	echo "Usage: $0 <environment-dir>"
	echo
	echo "Environments:"
	echo " * local/dev"
	echo " * aws/dev"
	echo " * aws/prod"
	exit 1
else
	env="$1"
	docker_dir="install/$env"

	echo "Building images for $docker_dir"
fi

# Check state
if [[ ${PWD##*/} != wikijump ]] || [[ ! -d .git ]]; then
	echo "We don't appear to be at the repo root!"
	exit 1
fi

echo "$docker_dir/docker-compose.yaml"
if [[ ! -f $docker_dir/docker-compose.yaml ]]; then
	echo "No docker-compose.yaml file found!"
	exit 1
fi

# Build images
for image in "${images[@]}"; do
	build_image "$env" "$image"
done
