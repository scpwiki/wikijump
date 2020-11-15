FROM ubuntu:focal

RUN mkdir /src
WORKDIR /src

# Publish HTTP

EXPOSE 80 443

# Build parameters

ARG WIKIJUMP_REPO="https://github.com/scpwiki/wikijump.git"
ARG WIKIJUMP_REPO_DIR="wikijump"
ARG WIKIJUMP_REPO_BRANCH="master"

ARG MAIN_DOMAIN="wikijump.test"
ARG FILES_DOMAIN="wjfiles.test"

ARG POSTGRES_VERSION="12"
ARG PHP_VERSION="7.4"
ARG XDIFF_VERSION="libxdiff-0.23"

# Derived variables

ARG PHP="php${PHP_VERSION}"
ARG WWW_DOMAIN="www.${MAIN_DOMAIN}"
ARG WIKIJUMP_DIR="/var/www/${WIKIJUMP_REPO_DIR}"

# Configure timezone

RUN ln -s /usr/share/zoneinfo/Etc/UTC /etc/localtime

# Install packages

RUN apt update
RUN apt install -y \
	'composer' \
	'git' \
	'html2text' \
	'imagemagick' \
	'memcached' \
	'nginx' \
	'openssl' \
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
	"postgresql-${POSTGRES_VERSION}" \
	'postgresql-client' \
	'postgresql-common' \
	'unzip' \
	'zip'

# Install xdiff

COPY scripts/*.sh ./scripts/
RUN bash scripts/install-xdiff.sh

# Upload files

COPY etc ./etc
COPY var ./var
COPY postgres-ingest.sql.gz ./

# Upload production scripts

RUN mkdir /app
COPY scripts/prod/* /app/
RUN sed -i "s/%PHP%/${PHP}/g" /app/services.sh

# Clone the Wikijump repository

WORKDIR /var/www

RUN git clone --branch "${WIKIJUMP_REPO_BRANCH}" "${WIKIJUMP_REPO}" "${WIKIJUMP_REPO_DIR}"
RUN mkdir -p "${WIKIJUMP_REPO_DIR}/web/tmp/smarty_templates_c"
RUN mkdir -p "${WIKIJUMP_REPO_DIR}/web/tmp/lucene_index"
RUN mkdir -p "${WIKIJUMP_REPO_DIR}/web/tmp/math"
RUN mkdir -p "${WIKIJUMP_REPO_DIR}/web/tmp/sitebackups"
RUN mkdir -p "${WIKIJUMP_REPO_DIR}/web/tmp/smarty_cache"
RUN mkdir -p "${WIKIJUMP_REPO_DIR}/web/tmp/smarty_macro_templates"
RUN mkdir -p "${WIKIJUMP_REPO_DIR}/web/tmp/htmlpurifier"

# Generate OpenSSL key
# See https://stackoverflow.com/a/41366949

WORKDIR /src
RUN openssl req \
	-x509 \
	-newkey rsa:4096 \
	-sha256 \
	-days 3650 \
	-nodes \
	-keyout cert.key \
	-out cert.crt \
	-subj "/CN=${MAIN_DOMAIN}" \
	-addext "subjectAltName=DNS:${FILES_DOMAIN},DNS:*.${MAIN_DOMAIN},DNS:*.${FILES_DOMAIN}"

RUN cat cert.crt cert.key > cert.pem

# Deploy generated key

RUN install -D -m644 cert.pem /usr/local/nginx/conf/cert.pem
RUN install -D -m644 cert.key /usr/local/nginx/conf/cert.key

# Import postgres data

RUN gunzip postgres-ingest.sql.gz
RUN sed -i "s/FILEDOMAIN/${FILES_DOMAIN}/g" postgres-ingest.sql
RUN scripts/setup-postgres.sh

# Install configuration files

RUN cp -av "./etc/nginx" "/etc"
RUN cp -av "./var/www/wikijump/conf/wikijump.ini" "${WIKIJUMP_DIR}/web/conf/wikijump.ini"
RUN cp -av "./etc/php/${PHP_VERSION}/cgi/conf.d/"* "/etc/php/${PHP_VERSION}/cgi"
RUN cp -av "./etc/php/${PHP_VERSION}/cgi/conf.d/"* "/etc/php/${PHP_VERSION}/fpm"

# Inject values in the wikijump.ini configuration file

RUN sed -i "s/BASEDOMAIN/${MAIN_DOMAIN}/g" "${WIKIJUMP_DIR}/web/conf/wikijump.ini"
RUN sed -i "s/MAINWIKI/${WWW_DOMAIN}/g" "${WIKIJUMP_DIR}/web/conf/wikijump.ini"
RUN sed -i "s/FILEDOMAIN/${FILES_DOMAIN}/g" "${WIKIJUMP_DIR}/web/conf/wikijump.ini"

# Update nginx configuration to point to the wikijump directory

RUN sed -i "s/wikijump/${WIKIJUMP_REPO_DIR}/g" /etc/nginx/sites-available/wikijump

# Enable wikijump site in nginx

RUN ln -sf ../sites-available/wikijump /etc/nginx/sites-enabled
RUN rm -f /etc/nginx/sites-enabled/default

# Run composer installation

WORKDIR "${WIKIJUMP_DIR}/web"
RUN rm -f composer.lock
RUN composer install
RUN chown -R www-data:www-data .
# Clean up

RUN rm -rf /src
WORKDIR /app

# Run server
CMD ["bash", "/app/run.sh"]
