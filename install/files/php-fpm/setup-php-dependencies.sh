#!/bin/bash
set -eux

cd /var/www/wikijump/web
su www-data \
	composer install \
		--no-ansi \
		--no-interaction \
		--no-scripts \
		--no-progress \
		--prefer-dist
