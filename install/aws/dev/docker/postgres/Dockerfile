FROM postgres:14-alpine

EXPOSE 5432

# Build variables
ARG ENVIRONMENT="dev"
ARG FILES_DOMAIN="wjfiles.test"
ARG PGDATA="/var/lib/postgresql/data"

# Install dependencies
RUN apk add --no-cache \
    postgresql-contrib \
    musl-locales

# Copy sources
COPY ./install/files/postgres /src
WORKDIR /src

# Create system user
RUN adduser -S wikijump

# Setup database
RUN /src/setup.sh

# Clean up
WORKDIR /etc/postgresql
RUN rm -rf /src

# Main process
USER postgres
CMD ["postgres"]
