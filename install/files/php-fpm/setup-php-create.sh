#!/bin/bash
set -eux

mkdir -p /var/www/wikijump/web
cd /var/www/wikijump/web

mkdir -p \
	bootstrap \
	conf \
	logs \
	storage/framework/cache \
	storage/framework/sessions \
	storage/framework/views \
	tmp/htmlpurifier \
	tmp/lucene_index \
	tmp/math \
	tmp/sitebackups \
	tmp/smarty_cache \
	tmp/smarty_macro_templates \
	tmp/smarty_templates_c \
	vendor

chown -R www-data:www-data \
	bootstrap/ \
	conf/ \
	logs/ \
	storage/ \
	tmp/ \
	vendor/
