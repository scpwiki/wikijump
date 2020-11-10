#!/bin/bash
set -e
set -o pipefail

# Variables and constants
readonly default_wikijump_repo='https://github.com/scpwiki/wikijump.git'
readonly default_main_domain='wikijump.test'
readonly default_files_domain='wjfiles.test'

readonly rootdir="$PWD"
readonly repodir='wikijump'

# Detect which init system the user is using
if [[ $(/sbin/init --version 2> /dev/null) =~ upstart ]]; then
	initsystem=upstart
elif [[ $(systemctl 2> /dev/null) =~ -\.mount ]]; then
	initsystem=systemd
elif [[ -f /etc/init.d/cron && ! -h /etc/init.d/cron ]]; then
	initsystem=sysv-init
else
	initsystem=unknown
fi

if [[ $initsystem = unknown ]]; then
	echo 'Unknown init system'
	exit 1
elif [[ $initsystem = upstart ]]; then
	echo 'UpStart not currently supported'
	exit 1
fi

# Get initial config
read -rp "Enter the link to the wikijump repo you wish to deploy [$default_wikijump_repo]: " repo
repo="${repo:-${default_wikijump_repo}}"

read -rp "Enter the domain name you want to use as the base for this deployment [$default_main_domain]: " basedomain
basedomain="${basedomain:-${default_main_domain}}"

read -rp "Enter the domain name you want to use as the files/uploads domain [$default_files_domain]: " filesdomain
filesdomain="${filesdomain:-${default_files_domain}}"

cd "$HOME"
sudo apt install -y \
	composer \
	html2text \
	imagemagick \
	memcached \
	nginx \
	php7.4 \
	php7.4-cgi \
	php7.4-cli \
	php7.4-common \
	php7.4-dev \
	php7.4-fpm \
	php7.4-gd \
	php7.4-memcache \
	php7.4-memcached \
	php7.4-pgsql \
	php7.4-tidy \
	php7.4-xdebug \
	php7.4-xml \
	postgresql-12 \
	postgresql-client \
	postgresql-common \
	unzip \
	zip

# On Ubuntu 18.04, will need to add a repository for PHP 7.4:
#sudo apt-get update && \
#	sudo add-apt-repository ppa:ondrej/php && \
#	sudo add-apt-repository ppa:ondrej/nginx && \
#	sudo apt-get update

# Enable and start our new services.
case "$initsystem" in
	systemd)
		sudo systemctl enable nginx --now
		sudo systemctl enable postgres --now
		sudo systemctl enable memcached --now
		sudo systemctl enable php7.4-fpm --now
		;;
	sysv-init)
		sudo service nginx start
		sudo service postgresql start
		sudo service memcached start
		sudo service php7.4-fpm start
		;;
esac

# libxdiff and xdiff
# PECL will error out if xdiff is already installed, so check first
if ! pecl list | grep -q xdiff; then
	wget 'http://www.xmailserver.org/libxdiff-0.23.tar.gz'
	tar -xzf libxdiff-0.23.tar.gz
	cd libxdiff-0.23/
	./configure
	sudo make
	sudo make install
	sudo pecl install xdiff
fi

# Generate OpenSSL Key - https://stackoverflow.com/a/41366949
# This requires openssl >=1.1.1 which is stock on Ubuntu 20.04
openssl req \
	-x509 \
	-newkey rsa:4096 \
	-sha256 \
	-days 3650 \
	-nodes \
	-keyout cert.key \
	-out cert.crt \
	-subj "/CN=$basedomain" \
	-addext "subjectAltName=DNS:$filesdomain,DNS:*.$basedomain,DNS:*.$filesdomain"
cat cert.crt cert.key > cert.pem

# Deploy key
sudo mkdir -p /usr/local/nginx/conf
sudo cp cert.pem /usr/local/nginx/conf
sudo cp cert.key /usr/local/nginx/conf

# Deploy wikidot from source
cd /var/www
sudo git clone "$repo" "$repodir"

# Import postgres DB
cd "$rootdir"
gunzip -k postgres-ingest.sql.gz
sed -i "s/FILEDOMAIN/$filesdomain/g" "$rootdir/postgres-ingest.sql"
sudo -u postgres sh -c "psql < '$rootdir/postgres-ingest.sql'"

# Copy config files
# sudo cp ./etc/lighttpd/* /etc/lighttpd
sudo cp -r "$rootdir/etc/nginx"/* /etc/nginx
sudo cp "$rootdir/var/www/wikijump/conf/wikijump.ini" "/var/www/$repodir/conf/wikijump.ini"
sudo cp "$rootdir/etc/php/7.4/cgi/conf.d"/* "/etc/php/7.4/cgi/conf.d"
sudo cp "$rootdir/etc/php/7.4/cgi/conf.d"/* "/etc/php/7.4/fpm/conf.d"

# Inject some values into our wikijump.ini file
mainwiki="www.$basedomain"
sudo sed -i "s/BASEDOMAIN/$basedomain/g" "/var/www/$repodir/conf/wikijump.ini"
sudo sed -i "s/MAINWIKI/$mainwiki/g" "/var/www/$repodir/conf/wikijump.ini"
sudo sed -i "s/FILEDOMAIN/$filesdomain/g" "/var/www/$repodir/conf/wikijump.ini"

# Update nginx config to point to right folder
sudo sed -i "s/wikijump/$repodir/g" "/etc/nginx/sites-available/wikijump"

# Link Wikijump and restart nginx
sudo ln -sf /etc/nginx/sites-available/wikijump /etc/nginx/sites-enabled
sudo rm -f /etc/nginx/sites-enabled/default
case "$initsystem" in
	systemd)
		sudo systemctl disable apache2 --now || true
		sudo systemctl restart nginx
		;;
	sysv-init)
		sudo service apache2 stop || true
		sudo service nginx restart
		;;
esac

# Reconfigure postgres
sudo sed -i 's/postgres\s*peer/postgres md5/g' /etc/postgresql/12/main/pg_hba.conf
echo "ALTER USER postgres WITH PASSWORD 'postgres';" | sudo -u postgres psql
case "$initsystem" in
	systemd)
		sudo systemctl restart postgresql
		;;
	sysv-init)
		sudo service postgresql restart
		;;
esac

# Deploy web
cd "/var/www/$repodir"
sudo composer install

# Modify permissions for temp folders
sudo chmod -R 777 tmp/smarty_templates_c

# vim set ft=bash ts=4 et:
