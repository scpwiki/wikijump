#!/bin/bash
set -eux

# Setup extensions
/src/setup-memcached.sh

# Configure PHP Intl
apt install -y libicu-dev
docker-php-ext-configure intl
docker-php-ext-install intl

# Configure PHP Data Structures
pecl install ds
docker-php-ext-enable ds
