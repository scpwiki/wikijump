#!/bin/bash
set -eux

install \
	-m 400 \
	-o www-data \
	-g www-data \
	.env.example .env

php artisan key:generate
