FROM ubuntu:focal

# Build variables
ARG WIKIJUMP_REPO="https://github.com/scpwiki/wikijump.git"
ARG WIKIJUMP_REPO_DIR="wikijump"
ARG WIKIJUMP_REPO_BRANCH="master"

ARG PHP_VERSION="7.4"
ARG XDIFF_VERSION="libxdiff-0.23"

ARG MAIN_DOMAIN="wikijump.test"
ARG FILES_DOMAIN="wjfiles.test"

# Derived variables
ARG PHP="php${PHP_VERSION}"
ARG WWW_DOMAIN="www.${MAIN_DOMAIN}"
ARG WIKIJUMP_DIR="/var/www/${WIKIJUMP_REPO_DIR}"

# Configure timezone
RUN ln -s /usr/share/zoneinfo/Etc/UTC /etc/localtime

# Preparation
RUN mkdir /src
WORKDIR /src

# Install packages
RUN apt update
RUN apt install -y \
	'composer' \
	'git' \
	'html2text' \
	'imagemagick' \
	'nodejs' \
	'npm' \
	"${PHP}" \
	"${PHP}-cgi" \
	"${PHP}-cli" \
	"${PHP}-common" \
	"${PHP}-dev" \
	"${PHP}-fpm" \
	"${PHP}-gd" \
	"${PHP}-memcache" \
	"${PHP}-memcached" \
	"${PHP}-pgsql" \
	"${PHP}-tidy" \
	"${PHP}-xdebug" \
	"${PHP}-xml" \
	'postgresql-client'

# Install xdiff
COPY install-xdiff.sh ./
RUN bash install-xdiff.sh

# Clone the Wikijump repository
WORKDIR /var/www
RUN git clone \
    --bare \
    --branch "${WIKIJUMP_REPO_BRANCH}" \
    "${WIKIJUMP_REPO}" "${WIKIJUMP_REPO_DIR}"

RUN mkdir -p "${WIKIJUMP_REPO_DIR}/web/tmp/smarty_templates_c"
RUN mkdir -p "${WIKIJUMP_REPO_DIR}/web/tmp/lucene_index"
RUN mkdir -p "${WIKIJUMP_REPO_DIR}/web/tmp/math"
RUN mkdir -p "${WIKIJUMP_REPO_DIR}/web/tmp/sitebackups"
RUN mkdir -p "${WIKIJUMP_REPO_DIR}/web/tmp/smarty_cache"
RUN mkdir -p "${WIKIJUMP_REPO_DIR}/web/tmp/smarty_macro_templates"
RUN mkdir -p "${WIKIJUMP_REPO_DIR}/web/tmp/htmlpurifier"

# Install configuration files
COPY etc ./
COPY wikijump.ini ./

RUN install -D -m644 "./wikijump.ini" "${WIKIJUMP_DIR}/web/conf/wikijump.ini"
RUN cp -av "./etc/php/${PHP_VERSION}/cgi/conf.d/"* "/etc/php/${PHP_VERSION}/cgi"
RUN cp -av "./etc/php/${PHP_VERSION}/cgi/conf.d/"* "/etc/php/${PHP_VERSION}/fpm"

# Inject values in the wikijump.ini configuration file
RUN sed -i "s/BASEDOMAIN/${MAIN_DOMAIN}/g" "${WIKIJUMP_DIR}/web/conf/wikijump.ini"
RUN sed -i "s/MAINWIKI/${WWW_DOMAIN}/g" "${WIKIJUMP_DIR}/web/conf/wikijump.ini"
RUN sed -i "s/FILEDOMAIN/${FILES_DOMAIN}/g" "${WIKIJUMP_DIR}/web/conf/wikijump.ini"

# Run composer installation
WORKDIR "${WIKIJUMP_DIR}/web"
RUN rm -f composer.lock
RUN composer install

# Run node installation and build
RUN npm install
RUN npm run build

# Clean up
WORKDIR /var/www
RUN chown -R www-data:www-data .
RUN rm -rf /src

# TODO
