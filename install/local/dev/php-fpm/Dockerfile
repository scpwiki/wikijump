#
# PHP
#

FROM php:7.4-fpm-buster AS php

# Build variables
ARG ENVIRONMENT="local/dev"
ARG DEBUG_IP="192.168.42.128"

ARG MAIN_DOMAIN="wikijump.test"
ARG FILES_DOMAIN="wjfiles.test"

# Derived variables
ARG BUILD_DIR="./install/${ENVIRONMENT}/php-fpm"
ARG WWW_DOMAIN="www.${MAIN_DOMAIN}"

# Copy source
RUN mkdir /src
COPY ./web /var/www/wikijump/web

# Configure xdebug
ARG XDEBUG_INI=/usr/local/etc/php/conf.d/docker-php-ext-xdebug.ini

# !! Put your IP for debugging here.
RUN echo "xdebug.mode = develop" >> ${XDEBUG_INI} \
 && echo "xdebug.client_host = ${DEBUG_IP}" >> ${XDEBUG_INI}

# Install composer from the official image
COPY --from=composer /usr/bin/composer /usr/bin/composer

# Copy setup scripts
COPY ${BUILD_DIR}/setup*.sh /src/

# TODO - let's see if we actually need xdiff and if so can we include it as an artifact or docker layer

# Run setup scripts
WORKDIR /var/www/wikijump/web
RUN /src/setup.sh

# Inject values in the wikijump.ini configuration file
COPY ${BUILD_DIR}/wikijump.ini conf/wikijump.ini

RUN sed -i "s/BASEDOMAIN/${MAIN_DOMAIN}/g" conf/wikijump.ini && \
    sed -i "s/MAINWIKI/${WWW_DOMAIN}/g" conf/wikijump.ini && \
    sed -i "s/FILEDOMAIN/${FILES_DOMAIN}/g" conf/wikijump.ini

# Copy remaining PHP files
COPY ${BUILD_DIR}/etc/php/ /usr/local/etc/php/conf.d/

#
# Frontend
#

FROM circleci/node:latest

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
RUN pnpm install
RUN pnpm build:legacy

#
# ftml
#

FROM rust:latest as ftml

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
# TODO - copy from client

WORKDIR /var/www/wikijump/web
ENTRYPOINT ["sh", "-c", "php artisan migrate && php-fpm"]
