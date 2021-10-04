#!/bin/bash
set -eux

# Variables
readonly cores="$(nproc)"

# Install dependencies
readonly dependencies=(
	'libmemcached-dev'
	'postgresql-server-dev-11'
)

apt install -y "${dependencies[@]}"

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

# Uninstall temporary dependencies
apt remove -y "${dependencies[@]}"

# Enable PHP extensions and clean up
docker-php-ext-enable igbinary memcached memcache xdebug
docker-php-ext-install \
	"-j$cores" \
		opcache \
		pgsql \
		pdo_pgsql \
		tidy \
		gd \
		gettext
