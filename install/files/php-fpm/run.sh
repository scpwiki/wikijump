#!/bin/bash
set -e

echo "Running migrations..."
php artisan migrate

echo "Starting the PHP-FPM daemon..."
php-fpm
