FROM postgres:14-alpine

EXPOSE 5432

# Build variables
ARG ENVIRONMENT="local"
ARG FILES_DOMAIN="wjfiles.test"

# Create system user
RUN adduser -S wikijump

# Install dependencies
RUN apk add --no-cache \
    postgresql-contrib \
    musl-locales

# Install entrypoint files
COPY ./install/files/postgres /docker-entrypoint-initdb.d
