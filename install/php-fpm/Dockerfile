FROM php:7.4-fpm-alpine

EXPOSE 80

# Build variables
ARG MAIN_DOMAIN="wikijump.test"
ARG FILES_DOMAIN="wjfiles.test"

# Derived variables
ARG WWW_DOMAIN="www.${MAIN_DOMAIN}"
ARG WIKIJUMP_DIR="/var/www/${WIKIJUMP_REPO_DIR}"

# Configure timezone
RUN ln -s /usr/share/zoneinfo/Etc/UTC /etc/localtime

# Preparation
RUN mkdir /src
WORKDIR /src

# Copy scripts
COPY setup-memcached.sh ./

# Enable wikijump site in nginx
RUN ln -sf /etc/nginx/sites-available/wikijump /etc/nginx/sites-enabled
RUN rm -f /etc/nginx/sites-enabled/default

# Install composer from the official image
COPY --from=composer /usr/bin/composer /usr/bin/composer

# Install packages
RUN apk add --update --no-cache \
    libgd \
    libpng-dev \
    libjpeg-turbo-dev \
    freetype-dev \
    imagemagick \
    nodejs \
    npm \
    html2text \
    nginx \
    libmemcached-libs \
    zlib \
    postgresql-dev \
    tidyhtml-dev

# Memcached PHP lib
RUN /src/setup-memcached.sh

# TODO - let's see if we actually need xdiff and if so can we include it as an artifact or docker layer

WORKDIR /var/www/wikijump/web

# Install nginx config files
COPY etc/nginx/* /etc/nginx/

# Install the Wikijump repository
WORKDIR /var/www
COPY ../../web /var/www
COPY wikijump.ini /var/www/web/conf/wikijump.ini

RUN mkdir -p \
    /var/www/web/tmp/smarty_templates_c \
    /var/www/web/tmp/lucene_index \
    /var/www/web/tmp/math \
    /var/www/web/tmp/sitebackups \
    /var/www/web/tmp/smarty_cache \
    /var/www/web/tmp/smarty_macro_templates \
    /var/www/web/tmp/htmlpurifier

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

# Main process
# Let the upstream ENTRYPOINT handle running php-fpm
CMD ["nginx"]
