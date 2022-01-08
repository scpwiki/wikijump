#
# PHP
#

FROM php:7.4-fpm-buster AS php

# Build variables
ARG ENVIRONMENT="prod"
ARG DEBUG_IP="192.168.42.128"

ARG MAIN_DOMAIN="wikijump.localhost"
ARG FILES_DOMAIN="wjfiles.localhost"

# Derived variables
ARG WWW_DOMAIN="www.${MAIN_DOMAIN}"

# Copy setup scripts
RUN mkdir /src
COPY ./install/files/php-fpm/setup*.sh /src/

# Install system dependencies
RUN /src/setup-system.sh

# Install composer from the official image
COPY --from=composer /usr/bin/composer /usr/bin/composer

# Install PHP extensions, dependencies
WORKDIR /var/www/wikijump/web
RUN /src/setup-php-create.sh
RUN /src/setup-php-extensions.sh

USER www-data
COPY ./web/lib/ /var/www/wikijump/web/lib
COPY ./web/composer.json ./web/composer.lock /var/www/wikijump/web/
RUN /src/setup-php-dependencies.sh

# Copy PHP configuration files
COPY ./install/files/php-fpm/etc/php/ /usr/local/etc/php/conf.d/

# Copy full Wikijump source
COPY ./web /var/www/wikijump/web

# Inject values in the wikijump.ini and .env configuration files
USER root
COPY ./install/files/${ENVIRONMENT}/wikijump.ini conf/wikijump.ini
COPY ./install/files/${ENVIRONMENT}/env .env

RUN sed -i "s/%%BASE_DOMAIN%%/${MAIN_DOMAIN}/g" conf/wikijump.ini && \
    sed -i "s/%%MAIN_WIKI%%/${WWW_DOMAIN}/g" conf/wikijump.ini && \
    sed -i "s/%%FILE_DOMAIN%%/${FILES_DOMAIN}/g" conf/wikijump.ini

#
# Rust
#

FROM rust:latest AS rust

# Copy source
RUN mkdir /src
COPY ./ftml /src/ftml
WORKDIR /src/ftml

# Download dependencies for caching
RUN cargo vendor

# Build ftml-ffi without logging
RUN cargo build \
    --release \
    --no-default-features \
    --features ffi,mathml

# Install mrml-cli for compiling MJML email templates
RUN cargo install --locked --version 1.3.2 mrml-cli

#
# Final image
#

FROM php
EXPOSE 9000

# Copy binaries from intermediate images
COPY --from=rust /src/ftml/target/release/ftml.h /usr/local/include/ftml.h
COPY --from=rust /src/ftml/target/release/libftml.so /usr/local/lib/libftml.so
COPY --from=rust /usr/local/cargo/bin/mrml /usr/local/bin/mrml

# PHP configuration preparation
RUN php artisan key:generate
RUN php artisan config:cache
RUN /src/setup-misc.sh

# Install runner
COPY ./install/files/php-fpm/run.sh /app/run.sh

# Cleanup
RUN rm -rf /src /usr/bin/composer

# We'll need to change the owner of the public directory,
# so that the user www-data can write to it.
RUN chown -R www-data /var/www/wikijump/web/public

USER www-data
ENTRYPOINT ["/app/run.sh"]
