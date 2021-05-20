#
# Build ftml for parsing and rendering
#

FROM rust:latest as rust

# Please note this arg is also defined in the php-fpm layer further down.
ARG WIKIJUMP_REPO_DIR="wikijump"

ARG WIKIJUMP_REPO="https://github.com/scpwiki/wikijump.git"
ARG WIKIJUMP_REPO_BRANCH="develop"
WORKDIR /src

RUN apt update
RUN apt install -y \
    build-essential \
    git

RUN git clone \
        --depth 10 \
        --branch "${WIKIJUMP_REPO_BRANCH}" \
        "${WIKIJUMP_REPO}" "${WIKIJUMP_REPO_DIR}"

WORKDIR /src/${WIKIJUMP_REPO_DIR}/ftml

# Build ftml-ffi without logging
RUN cargo build \
        --release \
        --no-default-features \
        --features ffi

#
# Main php-fpm container
#

FROM php:7.4-fpm-buster

EXPOSE 80

# Build variables
ARG WIKIJUMP_REPO_DIR="wikijump"

ARG MAIN_DOMAIN="wikijump.test"
ARG FILES_DOMAIN="wjfiles.test"

# Derived variables
ARG WWW_DOMAIN="www.${MAIN_DOMAIN}"
ARG WIKIJUMP_DIR="/var/www/${WIKIJUMP_REPO_DIR}"

# Preparation
RUN mkdir /src
WORKDIR /src

# Copy scripts
COPY setup-memcached.sh ./

# Install composer from the official image
COPY --from=composer /usr/bin/composer /usr/bin/composer

# Install packages
RUN apt update
RUN apt install -y \
    libgd-dev \
    libjpeg62-turbo-dev \
    libfreetype6-dev \
    imagemagick \
    git \
    zip \
    html2text \
    libmemcached-dev \
    postgresql-common \
    libtidy-dev \
    gettext && \

# Configure PHP-FFI
RUN apt install libffi-dev \
    && docker-php-ext-configure ffi --with-ffi \
    && docker-php-ext-install ffi

# Install libftml
COPY --from=rust /src/${WIKIJUMP_REPO_DIR}/ftml/target/release/ftml.h /usr/local/include/ftml.h
COPY --from=rust /src/${WIKIJUMP_REPO_DIR}/ftml/target/release/libftml.so /usr/local/lib/libftml.so

# Memcached PHP lib
RUN /src/setup-memcached.sh

# TODO - let's see if we actually need xdiff and if so can we include it as an artifact or docker layer

# Install the Wikijump repository
# Copy Wikijump files
COPY --from=rust /src/${WIKIJUMP_REPO_DIR}/web ${WIKIJUMP_DIR}/web

WORKDIR "${WIKIJUMP_DIR}/web"
RUN mkdir -p \
    tmp/smarty_templates_c \
    tmp/lucene_index \
    tmp/math \
    tmp/sitebackups \
    tmp/smarty_cache \
    tmp/smarty_macro_templates \
    tmp/htmlpurifier

# Inject values in the wikijump.ini configuration file
COPY wikijump.ini conf/wikijump.ini

RUN sed -i "s/BASEDOMAIN/${MAIN_DOMAIN}/g" conf/wikijump.ini
RUN sed -i "s/MAINWIKI/${WWW_DOMAIN}/g" conf/wikijump.ini
RUN sed -i "s/FILEDOMAIN/${FILES_DOMAIN}/g" conf/wikijump.ini

# Run composer install to install the dependencies
RUN composer install \
    --no-ansi \
    --no-interaction \
    --no-scripts \
    --no-progress \
    --prefer-dist

# Run NPM
RUN npm install && \
    npm run build

# Cleanup
RUN rm -rf /src
RUN chown -R www-data:www-data .

# Install nginx config files
COPY etc/nginx/ /etc/nginx/

# Enable wikijump site in nginx
RUN mkdir /etc/nginx/sites-enabled && \
    cp /etc/nginx/sites-available/wikijump /etc/nginx/sites-enabled/wikijump && \
    rm -f /etc/nginx/sites-enabled/default

RUN install -m 400 -o www-data -g www-data .env.example .env && php artisan key:generate

# Main process
# Let the upstream ENTRYPOINT handle running php-fpm
ADD ./entrypoint.sh /usr/local/bin/
CMD ["nginx", "-g", "daemon off;"]
ENTRYPOINT ["sh", "/usr/local/bin/entrypoint.sh"]
