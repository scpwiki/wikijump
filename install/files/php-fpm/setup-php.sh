#!/bin/bash
set -eux

# Setup extensions
/src/setup-memcached.sh

# Configure PHP-FFI
apt install -y libffi-dev
docker-php-ext-configure ffi --with-ffi
docker-php-ext-install ffi

# Configure PHP Intl
apt install -y libicu-dev
docker-php-ext-configure intl
docker-php-ext-install intl

# Install PHP dependencies
composer install \
	--no-ansi \
	--no-interaction \
	--no-scripts \
	--no-progress \
	--prefer-dist

# Cleanup
rm -f /usr/bin/composer
