#!/bin/bash
set -eux

cd /var/www/wikijump/web
composer install
	--no-ansi
	--no-interaction
	--no-scripts
	--no-progress
	--prefer-dist

php artisan key:generate
