#!/bin/bash
set -eux

install -m400 .env.example .env
php artisan key:generate
