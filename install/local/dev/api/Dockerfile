#
# DEEPWELL build
#

# This image is modified for development, it retains the
# full rust container and rebuilds as needed, to ease
# iteration during development.

FROM rust:latest AS rust

# Install system dependencies
RUN apt update
RUN apt install -y libmagic-dev

# Install helpers
RUN cargo install cargo-watch sqlx-cli

# Install files
COPY install/files/local/deepwell.toml /etc/deepwell.toml
COPY install/files/local/deepwell-start /bin/wikijump-deepwell-start
COPY install/files/api/health-check.sh /bin/wikijump-health-check

# /opt/locales is provided via docker-compose.dev.yaml

# Run migrations
#
# deepwell is able to do this itself, but we run it separately via
# the sqlx CLI tool to avoid dependency cycles wherein the deepwell
# build wants the database to be up, but the migrations running is
# dependent on the build having finished and deepwell running.
RUN sqlx migrate run

# Copy source
# Don't build until container execution (see cargo-watch)
RUN mkdir /src
COPY ./deepwell /src/deepwell
WORKDIR /src/deepwell

EXPOSE 2747
CMD ["/bin/wikijump-deepwell-start"]
