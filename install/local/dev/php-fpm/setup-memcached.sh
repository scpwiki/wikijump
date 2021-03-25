#!/bin/sh
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

apk add \
	--no-cache \
	--update \
	--virtual \
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

# Install memcache
pecl install --nobuild memcache-4.0.5.2
cd "$(pecl config-get temp_dir)/memcache"

phpize

./configure
make "-j$cores"
make install

# Install xdebug
pecl install xdebug

# Enable PHP extensions and clean up
docker-php-ext-enable igbinary memcached memcache xdebug
apk del .memcached-deps .phpize-deps
docker-php-ext-install \
	"-j$cores" \
		opcache \
		pgsql \
		tidy \
		gd \
		gettext
