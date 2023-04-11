#
# DEEPWELL build
#

# This image is modified for development, it retains the
# full rust container and rebuilds as needed, to ease
# iteration during development.

FROM rust:latest AS rust

# Install system dependencies
RUN apt update
RUN apt install libmagic-dev

# Install helpers
RUN cargo install cargo-watch

# Install development configuration file
COPY install/files/local/deepwell.toml /etc/deepwell.toml

# Copy source
# Don't build until container execution (see cargo-watch)
RUN mkdir /src
COPY ./deepwell /src/deepwell
WORKDIR /src/deepwell

CMD ["/usr/local/cargo/bin/cargo", "watch", "--why", "-x", "run -- /etc/deepwell.toml"]
