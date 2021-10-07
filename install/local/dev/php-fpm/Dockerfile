#
# PHP
#

FROM php:7.4-fpm-buster AS php

# Build variables
ARG ENVIRONMENT="local"
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

# !! Configure xdebug
ARG XDEBUG_INI=/usr/local/etc/php/conf.d/docker-php-ext-xdebug.ini

# !! Put your IP for debugging here.
RUN echo "xdebug.mode = develop" >> ${XDEBUG_INI} \
 && echo "xdebug.client_host = ${DEBUG_IP}" >> ${XDEBUG_INI}

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

# Copy full Wikijump source
COPY ./web /var/www/wikijump/web

# Generate a new application key
USER root
RUN php artisan key:generate

# TODO - let's see if we actually need xdiff and if so can we include it as an artifact or docker layer

# Inject values in the wikijump.ini configuration file
COPY ./install/files/${ENVIRONMENT}/wikijump.ini conf/wikijump.ini

RUN sed -i "s/BASEDOMAIN/${MAIN_DOMAIN}/g" conf/wikijump.ini && \
    sed -i "s/MAINWIKI/${WWW_DOMAIN}/g" conf/wikijump.ini && \
    sed -i "s/FILEDOMAIN/${FILES_DOMAIN}/g" conf/wikijump.ini

# Copy remaining PHP files
COPY ./install/files/php-fpm/etc/ /usr/local/etc/php/conf.d/

# Finish other miscellaneous setup
RUN /src/setup-misc.sh

#
# Frontend
#

FROM circleci/node:latest AS frontend

# Copy source
RUN sudo mkdir /src
COPY ./ftml /src/ftml
COPY ./locales /src/locales
COPY ./web /src/web
RUN sudo chown -R circleci /src

# Install pnpm
RUN sudo npm install -g pnpm

# Build frontend
WORKDIR /src/web
RUN pnpm install --prod
RUN pnpm build-container

#
# ftml
#

FROM rust:latest AS ftml

# Copy source
RUN mkdir /src
COPY ./ftml /src/ftml
WORKDIR /src/ftml

# Build ftml-ffi without logging
RUN cargo build \
        --release \
        --no-default-features \
        --features ffi,mathml

#
# Localizations
#

FROM python:3.9-slim AS locales

# Copy source
RUN mkdir /src
COPY ./locales /src/locales
WORKDIR /src/locales

# Install dependencies
RUN apt update && \
    apt install -y gettext
RUN pip3 install --upgrade -r requirements.txt

# Build localization files
RUN python3 -m messages .

#
# Final image
#

FROM php
EXPOSE 9000

# Copy sources from intermediate images
COPY --from=ftml /src/ftml/target/release/ftml.h /usr/local/include/ftml.h
COPY --from=ftml /src/ftml/target/release/libftml.so /usr/local/lib/libftml.so
COPY --from=locales /src/locales/out/ /var/www/locales
COPY --from=frontend /src/web/web/files--common/ /var/www/wikijump/web/web/files--common
COPY --from=frontend /src/web/public/ /var/www/wikijump/web/public

# Cleanup
RUN rm -rf /src /usr/bin/composer

USER www-data
ENTRYPOINT ["sh", "-c", "php artisan migrate && php-fpm"]
