# NOTE: when we migrate in-app instead of docker,
# we can replace this Dockerfile with just a postgres:12-alpine image
FROM postgres:12

EXPOSE 5432

# Build variables
ARG FILES_DOMAIN="wjfiles.test"
ARG PGDATA="/var/lib/postgresql/data"

ENV POSTGRES_CLUSTER="main"
ENV POSTGRES_VERSION="12"

# Preparation
RUN mkdir /src
WORKDIR /src

# Copy sources
COPY seed.sql setup.sh ./

# Install scripts
RUN mkdir /app
COPY run.sh /app

# Create system user
RUN useradd wikijump --create-home --shell /bin/bash

# Setup database
USER postgres
RUN pg_createcluster --user postgres --locale en_US.utf8 "${POSTGRES_VERSION}" "${POSTGRES_CLUSTER}"

# Install postgres auth configuration
COPY pg_hba.conf "/etc/postgresql/${POSTGRES_VERSION}/${POSTGRES_CLUSTER}/pg_hba.conf"

# Seed database
USER root
RUN ./setup.sh

# Clean up
WORKDIR /etc/postgresql
RUN rm -rf /src

# Main process
CMD ["/app/run.sh"]
