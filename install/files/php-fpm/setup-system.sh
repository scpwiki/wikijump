#!/bin/bash
set -eux

apt update
apt install -y \
	libgd-dev \
	libjpeg62-turbo-dev \
	libfreetype6-dev \
	imagemagick \
	git \
	zip \
	html2text \
	postgresql-common \
	libtidy-dev
