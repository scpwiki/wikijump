FROM php:7.4-fpm-alpine

EXPOSE 80

# Build variables
ARG MAIN_DOMAIN="wikijump.test"
ARG FILES_DOMAIN="wjfiles.test"

# Configure timezone
RUN ln -s /usr/share/zoneinfo/Etc/UTC /etc/localtime

# Preparation
RUN mkdir /src
WORKDIR /src

# Copy sources
COPY setup-memcached.sh ./

COPY etc/nginx/* /etc/nginx
COPY web /var/www/wikijump/web
COPY wikijump.ini /var/www/wikijump/web/conf/wikijump.ini

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

# Make temp dirs
RUN mkdir -p tmp/smarty_templates_c && \
    mkdir -p tmp/lucene_index && \
    mkdir -p tmp/math && \
    mkdir -p tmp/sitebackups && \
    mkdir -p tmp/smarty_cache && \
    mkdir -p tmp/smarty_macro_templates && \
    mkdir -p tmp/htmlpurifier

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
