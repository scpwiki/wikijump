# NOTE: when we migrate in-app instead of docker,
# we can replace this Dockerfile with just a postgres:12-alpine image
FROM postgres:12

ARG FILES_DOMAIN

# Preparation
RUN mkdir /src
WORKDIR /src

# Copy sources
COPY seed.sql.gz ./
COPY seed.sh ./

# Create cluster
RUN pg_createcluster -u postgres 12 main

# Seed database
RUN ./seed.sh

# Clean up
WORKDIR /etc/postgresql
RUN rm -rf /src

# Main process
EXPOSE 5432
ENTRYPOINT ["docker-entrypoint.sh"]
CMD ["postgres"]
