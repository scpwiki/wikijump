#!/bin/bash
set -eux

cd /var/www/wikijump/web

mkdir -p \
	storage/framework/cache \
	storage/framework/sessions \
	storage/framework/views \
	vendor \
	tmp/smarty_templates_c \
	tmp/lucene_index \
	tmp/math \
	tmp/sitebackups \
	tmp/smarty_cache \
	tmp/smarty_macro_templates \
	tmp/htmlpurifier

chown -R www-data:www-data \
	bootstrap/ \
	conf/ \
	logs/ \
	storage/ \
	tmp/ \
	vendor/
