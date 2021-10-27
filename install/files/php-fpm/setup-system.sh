#!/bin/bash
set -eux

apt update
apt install -y \
	git \
	html2text \
	imagemagick \
	libfreetype6-dev \
	libgd-dev \
	libjpeg62-turbo-dev \
	libtidy-dev \
	postgresql-common \
	zip
