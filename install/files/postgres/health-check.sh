#!/bin/sh

# Database is running
sudo -u wikijump pg_isready -d wikijump

# Database is accessible via network
sudo -u wikijump pg_isready -d wikijump -h localhost -p 5432
