#!/bin/bash
set -eux

[[ $USER == www-data ]]

cd /var/www/wikijump/web

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

install -m 400 .env.example .env
php artisan key:generate
