#!/bin/bash
set -eu

echo "PHP version: $PHP_VERSION"
echo
echo "Running php-fpm in foreground..."

exec "php-fpm${PHP_VERSION}" -F
