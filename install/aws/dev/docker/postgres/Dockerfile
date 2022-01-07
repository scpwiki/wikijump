FROM postgres:14-alpine

EXPOSE 5432

# Build variables
ARG ENVIRONMENT="dev"
ARG FILES_DOMAIN="wjfiles.test"

# Create system user
RUN adduser -S wikijump

# Install dependencies
RUN apk add --no-cache \
    musl-locales \
    postgresql-contrib \
    sudo

# Install files
COPY ./install/files/postgres/init /docker-entrypoint-initdb.d
COPY ./install/files/postgres/health-check.sh /bin/wikijump-health-check
