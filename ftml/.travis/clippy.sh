#!/bin/bash
set -eu

version="nightly-2020-01-02"
toolchain="$version-x86_64-unknown-linux-gnu"

if [[ $TRAVIS_OS_NAME != linux ]]; then
	echo 'Skipping build, only running on linux'
	exit 0
fi

case "$1" in
	setup)
		rustup toolchain install "$toolchain"
		rustup component add clippy --toolchain "$toolchain"
		;;
	check)
		cargo "+$version" clippy
		;;
	*)
		echo "Unknown job: $1"
		exit 1
esac
