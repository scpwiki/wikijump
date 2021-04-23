FROM php:7.4-fpm-alpine

EXPOSE 9000

# Build variables
ARG WIKIJUMP_REPO="https://github.com/scpwiki/wikijump.git"
ARG WIKIJUMP_REPO_DIR="wikijump"
ARG WIKIJUMP_REPO_BRANCH="develop"
ARG DEBUG_IP="192.168.42.128"

ARG MAIN_DOMAIN="wikijump.test"
ARG FILES_DOMAIN="wjfiles.test"

# Derived variables
ARG WWW_DOMAIN="www.${MAIN_DOMAIN}"
ARG WIKIJUMP_DIR="/var/www/${WIKIJUMP_REPO_DIR}"

# Configure timezone
RUN ln -s /usr/share/zoneinfo/Etc/UTC /etc/localtime

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
    git \
    html2text \
    libmemcached-libs \
    zlib \
    postgresql-dev \
    tidyhtml-dev \
    gettext-dev && \

    # Memcached PHP lib
    /src/setup-memcached.sh

ARG XDEBUG_INI=/usr/local/etc/php/conf.d/docker-php-ext-xdebug.ini

# !! Put your IP for debugging here.
RUN echo "xdebug.mode = debug" >> ${XDEBUG_INI} \
 && echo "xdebug.client_host = ${DEBUG_IP}" >> ${XDEBUG_INI}

# TODO - let's see if we actually need xdiff and if so can we include it as an artifact or docker layer

# Install the Wikijump repository
WORKDIR /var/www

# !! Workaround for WSL2-enabled development with exposed host volumes, comment out if not needed.
RUN apk add shadow && usermod -u 1000 www-data && groupmod -g 1000 www-data && \

    git clone \
        --depth 10 \
        --branch "${WIKIJUMP_REPO_BRANCH}" \
        "${WIKIJUMP_REPO}" "${WIKIJUMP_REPO_DIR}"

WORKDIR "${WIKIJUMP_DIR}/web"

# Inject values in the wikijump.ini configuration file
COPY wikijump.ini conf/wikijump.ini

RUN sed -i "s/BASEDOMAIN/${MAIN_DOMAIN}/g" conf/wikijump.ini && \
    sed -i "s/MAINWIKI/${WWW_DOMAIN}/g" conf/wikijump.ini && \
    sed -i "s/FILEDOMAIN/${FILES_DOMAIN}/g" conf/wikijump.ini && \

    # Run composer install to install the dependencies
    composer install \
        --no-ansi \
        --no-interaction \
        --no-scripts \
        --no-progress \
        --prefer-dist && \

    # Cleanup
    rm -rf /src && \
    rm -f /usr/bin/composer && \

    # Create tmp folders
    mkdir -p \
        tmp/smarty_templates_c \
        tmp/lucene_index \
        tmp/math \
        tmp/sitebackups \
        tmp/smarty_cache \
        tmp/smarty_macro_templates \
        tmp/htmlpurifier && \

# Enable wikijump site in nginx
    mkdir -p /var/log/php && \
    touch /var/log/php/fpm-error.log && \
    echo "access.format = \"[%t] %m %{REQUEST_SCHEME}e://%{HTTP_HOST}e%{REQUEST_URI}e %f pid:%p took:%ds mem:%{mega}Mmb cpu:%C%% status:%s {%{HTTP_X_REAL_IP}e|%{HTTP_USER_AGENT}e}\"" >> /usr/local/etc/php-fpm.d/docker.conf && \

    install -m 400 -o www-data -g www-data .env.example .env && \
    php artisan key:generate && \
    chown -R www-data:www-data .

COPY etc/php/ /usr/local/etc/php/conf.d/

CMD ["php-fpm"]
