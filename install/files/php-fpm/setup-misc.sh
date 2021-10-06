#!/bin/bash
set -eux

# Change current directory
cd /var/www/wikijump/web

# Create tmp folders
mkdir -p \
	storage/framework/cache \
	storage/framework/sessions \
	storage/framework/views \
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
chown -R www-data:www-data web/{logs,storage,tmp}
