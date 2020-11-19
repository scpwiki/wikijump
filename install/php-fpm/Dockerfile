FROM php:7.4-fpm-alpine

EXPOSE 80

# TODO: copy from bluesoul's container Dockerfile

# Build variables
ARG WIKIJUMP_REPO="https://github.com/scpwiki/wikijump.git"
ARG WIKIJUMP_REPO_DIR="wikijump"
ARG WIKIJUMP_REPO_BRANCH="master"

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
    git \
    html2text \
    nginx \
    libmemcached-libs \
    zlib \
    postgresql-dev \
    tidyhtml-dev

# Memcached PHP lib
RUN /src/setup-memcached.sh

# TODO - let's see if we actually need xdiff and if so can we include it as an artifact or docker layer

# Install nginx config files
COPY etc/nginx/* /etc/nginx/

# Enable wikijump site in nginx
RUN ln -sf /etc/nginx/sites-available/wikijump /etc/nginx/sites-enabled
RUN rm -f /etc/nginx/sites-enabled/default

# Install the Wikijump repository
WORKDIR /var/www
RUN git clone \
    --depth 10 \
    --branch "${WIKIJUMP_REPO_BRANCH}" \
    "${WIKIJUMP_REPO}" "${WIKIJUMP_REPO_DIR}"

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

# Main process
# Let the upstream ENTRYPOINT handle running php-fpm
CMD ["nginx", "-g", "daemon off;"]
