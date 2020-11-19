#!/bin/bash
set -eux

# Variables
readonly cores="$(nproc)"

# Install dependencies
apk add \
	--no-cache \
	--update \
	--virtual \
		.phpize-deps \
		$PHPIZE_DEPS \
		.memcached-deps \
		zlib-dev \
		libmemcached-dev \
		cyrus-sasl-dev

# Install igbinary (memcached's deps)
pecl install igbinary

# Install memcached
pecl install --nobuild memcached
cd "$(pecl config-get temp_dir)/memcached"

phpize

./configure --enable-memcached-igbinary
make "-j$cores"
make install

# Enable PHP extensions and clean up
docker-php-ext-enable igbinary memcached
apk del .memcached-deps .phpize-deps
docker-php-ext-install \
	"-j$cores" \
		opcache
		pgsql
		tidy
		gd
