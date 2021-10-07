#!/bin/bash
set -eux

install -m 400 .env.example .env
php artisan key:generate
