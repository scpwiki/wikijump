FROM postgres:12-alpine

EXPOSE 5432

# Preparation
RUN mkdir /src
WORKDIR /src

# Copy sources
COPY ingest.sql.gz ./
COPY setup.sh ./

# Setup database
RUN gunzip ingest.sql.gz
RUN sed -i "s/FILEDOMAIN/${FILES_DOMAIN}/g" ingest.sql
RUN ./setup.sh
