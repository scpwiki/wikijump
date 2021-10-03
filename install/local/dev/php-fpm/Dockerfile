#
# Frontend
#

FROM circleci/node:latest

# Copy source
RUN sudo mkdir /src
COPY ./client /src/client
COPY ./ftml /src/ftml
COPY ./locales /src/locales
COPY ./web /src/web
RUN sudo chown -R circleci /src

# Install pnpm
RUN sudo npm install -g pnpm

# Build main frontend
WORKDIR /src/client
RUN pnpm install
RUN pnpm build

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

FROM php:7.4-fpm-buster
EXPOSE 9000

# Build variables
ARG ENVIRONMENT="local/dev"
ARG WIKIJUMP_DIR="/var/www/wikijump"
ARG DEBUG_IP="192.168.42.128"

ARG MAIN_DOMAIN="wikijump.test"
ARG FILES_DOMAIN="wjfiles.test"

# Derived variables
ARG BUILD_DIR="./install/${ENVIRONMENT}/php-fpm"
ARG WWW_DOMAIN="www.${MAIN_DOMAIN}"

# Copy Wikijump repository
COPY ./web ${WIKIJUMP_DIR}/web
RUN mkdir /src

# Copy sources from intermediate images
COPY --from=ftml /src/ftml/target/release/ftml.h /usr/local/include/ftml.h
COPY --from=ftml /src/ftml/target/release/libftml.so /usr/local/lib/libftml.so
COPY --from=locales /src/locales/out/ /var/www/locales
# TODO - copy from client

# Copy setup scripts
COPY ${BUILD_DIR}/setup*.sh /src/

# Install composer from the official image
COPY --from=composer /usr/bin/composer /usr/bin/composer

# TODO - let's see if we actually need xdiff and if so can we include it as an artifact or docker layer

# Configure xdebug
ARG XDEBUG_INI=/usr/local/etc/php/conf.d/docker-php-ext-xdebug.ini

# !! Put your IP for debugging here.
RUN echo "xdebug.mode = develop" >> ${XDEBUG_INI} \
 && echo "xdebug.client_host = ${DEBUG_IP}" >> ${XDEBUG_INI}

# !! Workaround for WSL2-enabled development with exposed host volumes, comment out if not needed.
# RUN usermod -u 1000 www-data && groupmod -g 1000 www-data
RUN chown -R www-data:www-data .

# Setup PHP
WORKDIR "${WIKIJUMP_DIR}/web"

# Inject values in the wikijump.ini configuration file
COPY ${BUILD_DIR}/wikijump.ini conf/wikijump.ini

RUN sed -i "s/BASEDOMAIN/${MAIN_DOMAIN}/g" conf/wikijump.ini && \
    sed -i "s/MAINWIKI/${WWW_DOMAIN}/g" conf/wikijump.ini && \
    sed -i "s/FILEDOMAIN/${FILES_DOMAIN}/g" conf/wikijump.ini

# Run setup scripts
RUN /src/setup.sh

# Copy remaining PHP files
COPY ${BUILD_DIR}/etc/php/ /usr/local/etc/php/conf.d/

# Cleanup staging files
RUN rm -rf /src

ENTRYPOINT ["sh", "-c", "php artisan migrate && php-fpm"]
