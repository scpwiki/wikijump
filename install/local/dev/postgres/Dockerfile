FROM postgres:13-alpine

EXPOSE 5432

# Build variables
ARG ENVIRONMENT="local"
ARG FILES_DOMAIN="wjfiles.test"
ARG PGDATA="/var/lib/postgresql/data"

# Install dependencies
RUN apk add --no-cache musl-locales

# Copy sources
COPY ./install/files/postgres /src
WORKDIR /src

# Create system user
RUN adduser -S wikijump

# Setup database
USER postgres
RUN initdb -D ${PGDATA} -A trust --locale=en_US.UTF-8

# Install postgres auth configuration
RUN install -D -m644 /src/pg_hba.conf "${PGDATA}/pg_hba.conf"

# Seed database
USER root
RUN /src/setup.sh

# Clean up
WORKDIR /etc/postgresql
RUN rm -rf /src

# Main process
USER postgres
CMD ["postgres"]
