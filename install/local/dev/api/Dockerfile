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
RUN cargo install cargo-watch

# Install files
COPY install/files/local/deepwell.toml /etc/deepwell.toml
COPY install/files/api/health-check.sh /bin/wikijump-health-check

# /opt/locales is provided via docker-compose.dev.yaml

# Copy source
# Don't build until container execution (see cargo-watch)
RUN mkdir /src
COPY ./deepwell /src/deepwell
WORKDIR /src/deepwell

EXPOSE 2747
CMD ["/usr/local/cargo/bin/cargo", "watch", "-w", "/src/deepwell", "-w", "/opt/locales", "-w", "/etc/deepwell.toml", "--why", "-x", "run -- /etc/deepwell.toml"]
