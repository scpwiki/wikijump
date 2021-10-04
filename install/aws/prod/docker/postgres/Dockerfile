FROM postgres:13

EXPOSE 5432

# Build variables
ARG ENVIRONMENT="pro"
ARG FILES_DOMAIN="wjfiles.test"
ARG PGDATA="/var/lib/postgresql/data"

ENV POSTGRES_CLUSTER="main"
ENV POSTGRES_VERSION="13"

# Copy sources
COPY ./install/files/postgres /src
WORKDIR /src

# Install script
COPY run.sh /app

# Create system user
RUN useradd wikijump --create-home --shell /bin/bash

# Setup database
USER postgres
RUN pg_createcluster --user postgres --locale en_US.UTF-8 "${POSTGRES_VERSION}" "${POSTGRES_CLUSTER}"

# Install postgres auth configuration
COPY pg_hba.conf "/etc/postgresql/${POSTGRES_VERSION}/${POSTGRES_CLUSTER}/pg_hba.conf"

# Seed database
USER root
RUN /src/setup.sh

# Clean up
WORKDIR /etc/postgresql
RUN rm -rf /src

# Main process
CMD ["/app/run.sh"]
