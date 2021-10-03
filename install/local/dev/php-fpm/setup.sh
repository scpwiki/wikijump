#!/bin/bash
set -eux

# Install system dependencies
apt update
apt install -y \
	libgd-dev \
	libjpeg62-turbo-dev \
	libfreetype6-dev \
	imagemagick \
	git \
	zip \
	html2text \
	postgresql-common \
	libtidy-dev \
	gettext

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
rm -rf /src
rm -f /usr/bin/composer

# Create tmp folders
mkdir -p \
	storage \
	tmp/smarty_templates_c \
	tmp/lucene_index \
	tmp/math \
	tmp/sitebackups \
	tmp/smarty_cache \
	tmp/smarty_macro_templates \
	tmp/htmlpurifier

# Enable wikijump site in nginx
mkdir -p /var/log/php
touch /var/log/php/fpm-error.log
echo "access.format = \"[%t] %m %{REQUEST_SCHEME}e://%{HTTP_HOST}e%{REQUEST_URI}e %f pid:%p took:%ds mem:%{mega}Mmb cpu:%C%% status:%s {%{HTTP_X_REAL_IP}e|%{HTTP_USER_AGENT}e}\"" >> /usr/local/etc/php-fpm.d/docker.conf

install -m 400 -o www-data -g www-data .env.example .env
php artisan key:generate
chown -R www-data:www-data .
