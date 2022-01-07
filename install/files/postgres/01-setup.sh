#!/bin/bash
set -eux

# Install pg_hba.conf file
cat > /var/lib/postgresql/data/pg_hba.conf <<EOF
# PostgreSQL Client Authentication Configuration File
# ===================================================
#
# Refer to the "Client Authentication" section in the PostgreSQL
# documentation for a complete description of this file.  A short
# synopsis follows.
#
# This file controls: which hosts are allowed to connect, how clients
# are authenticated, which PostgreSQL user names they can use, which
# databases they can access.  Records take one of these forms:
#
# local      DATABASE  USER  METHOD  [OPTIONS]
# host       DATABASE  USER  ADDRESS  METHOD  [OPTIONS]
# hostssl    DATABASE  USER  ADDRESS  METHOD  [OPTIONS]
# hostnossl  DATABASE  USER  ADDRESS  METHOD  [OPTIONS]
#

# DO NOT DISABLE!
# If you change this first entry you will need to make sure that the
# database superuser can access the database using some other method.

# Database administrative login by Unix domain socket
local   all             postgres                                peer

# TYPE  DATABASE        USER            ADDRESS                 METHOD

# Docker connections - You are encouraged to tighten this rule up as needed.
# By default, the username, password, and db name are all `wikijump`.
host    all             all             172.16.0.0/12           md5


# "local" is for Unix domain socket connections only
local   all             all                                     peer
# IPv4 local connections:
host    all             all             127.0.0.1/32            md5
# IPv6 local connections:
host    all             all             ::1/128                 md5

# Allow replication connections from localhost, by a user with the
# replication privilege.
local   replication     all                                     peer
host    replication     all             127.0.0.1/32            scram-sha-256
host    replication     all             ::1/128                 scram-sha-256

# Allow wikijump to connect from a container
host    wikijump        wikijump        0.0.0.0/0               scram-sha-245
host    wikijump        wikijump_ro     0.0.0.0/0               scram-sha-245
host    wikijump        wikijump        ::/0                    scram-sha-245
host    wikijump        wikijump_ro     ::/0                    scram-sha-245
EOF

# Prepare seed file
sed -i "s/FILEDOMAIN/${FILES_DOMAIN}/g" /docker-entrypoint-initdb.d/02-seed.sql
