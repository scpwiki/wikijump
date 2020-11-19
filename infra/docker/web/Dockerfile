FROM php:7.4-fpm-alpine

EXPOSE 80

# Create some environment variables if not present.
# TBD - We're going to be injecting a lot of values via GitHub Actions.

# Configure timezone
RUN ln -s /usr/share/zoneinfo/Etc/UTC /etc/localtime

# Install Libraries
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

# Install PHP Extensions not already present in alpine release
# RUN docker-php-ext-install -j$(nproc) \
#     opcache \
#     pgsql \
#     tidy \
#     gd

# Memcached PHP lib
RUN set -xe && \
    cd /tmp/ && \
    apk add --no-cache --update --virtual .phpize-deps $PHPIZE_DEPS && \
    apk add --no-cache --update --virtual .memcached-deps zlib-dev libmemcached-dev cyrus-sasl-dev && \
# Install igbinary (memcached's deps)
    pecl install igbinary && \
# Install memcached
    ( \
        pecl install --nobuild memcached && \
        cd "$(pecl config-get temp_dir)/memcached" && \
        phpize && \
        ./configure --enable-memcached-igbinary && \
        make -j$(nproc) && \
        make install && \
        cd /tmp/ \
    ) && \
# Enable PHP extensions
    docker-php-ext-enable igbinary memcached && \
    rm -rf /tmp/* && \
    apk del .memcached-deps .phpize-deps && \
    docker-php-ext-install -j$(nproc) \
    opcache \
    pgsql \
    tidy \
    gd

# Copy PHP config - let's see if we actually need xdiff and if so can we include it as an artifact or docker layer

# Copy web files to web root.
ADD web /var/www/wikijump/web

# Copy nginx config
ADD installer/etc/nginx/* /etc/nginx

# Copy wikijump config
ADD installer/var/www/wikijump/conf/wikijump.ini /var/www/wikijump/web/conf/wikijump.ini

# Enable wikijump site in nginx
RUN ln -sf /etc/nginx/sites-available/wikijump /etc/nginx/sites-enabled && \
    rm -f /etc/nginx/sites-enabled/default

# Install composer from the official image
COPY --from=composer /usr/bin/composer /usr/bin/composer

WORKDIR /var/www/wikijump/web

# Make temp dirs
RUN mkdir -p tmp/smarty_templates_c && \
    mkdir -p tmp/lucene_index && \
    mkdir -p tmp/math && \
    mkdir -p tmp/sitebackups && \
    mkdir -p tmp/smarty_cache && \
    mkdir -p tmp/smarty_macro_templates && \
    mkdir -p tmp/htmlpurifier

# Run composer install to install the dependencies
RUN composer install --no-ansi --no-interaction --no-scripts --no-progress --prefer-dist

# Run NPM
RUN npm install && \
    npm run build

# Run nginx, let the upstream ENTRYPOINT handle running php-fpm
CMD nginx
