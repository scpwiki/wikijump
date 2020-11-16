# NOTE: when we migrate in-app instead of docker,
# we can replace this Dockerfile with just a postgres:12-alpine image
FROM postgres:12

EXPOSE 5432

# Preparation
RUN mkdir /src
WORKDIR /src

# Copy sources
COPY ingest.sql.gz ./
COPY setup.sh ./

# Setup database
RUN gunzip ingest.sql.gz
RUN sed -i "s/FILEDOMAIN/${files_domain}/g" ingest.sql
RUN ./setup.sh
