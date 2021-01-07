#!/bin/bash
set -eux

if pecl list | grep -q xdiff; then
	echo 'xdiff is already installed!'
	exit
fi

# Get variables from the Dockerfile

readonly version="${XDIFF_VERSION}"

# Create dir

mkdir xdiff
cd xdiff

# Build
curl "http://www.xmailserver.org/$version.tar.gz" -o "$version.tar.gz"
tar -xzvf "$version.tar.gz"
cd "$version/"
./configure
make

# Install
make install
pecl install xdiff
