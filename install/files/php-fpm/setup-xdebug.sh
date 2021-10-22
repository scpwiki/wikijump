#!/bin/bash
set -eux

pecl install xdebug
docker-php-ext-enable xdebug
