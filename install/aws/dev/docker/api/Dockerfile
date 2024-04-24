#
# DEEPWELL build
#

FROM rust:alpine AS rust

# Install system dependencies
RUN apk update
RUN apk add --no-cache build-base libmagic libmagic-static

# Copy source
RUN mkdir /src
COPY ./deepwell /src/deepwell
WORKDIR /src/deepwell

# Cache rust dependencies
RUN cargo vendor

# Build deepwell server
RUN cargo build --release

#
# Final image
#

# We want alpine, but need glibc
FROM alpine:latest

ENV LOCALIZATION_PATH="/opt/locales"

RUN apk update
RUN apk add --no-cache curl
COPY --from=rust /src/deepwell/target/release/deepwell /usr/local/bin/deepwell
COPY ./install/files/api/health-check.sh /bin/wikijump-health-check
COPY ./install/files/dev/deepwell.toml /etc/deepwell.toml
COPY ./locales/fluent /opt/locales/fluent
COPY ./deepwell/seeder /opt/seeder

USER daemon
CMD ["/usr/local/bin/deepwell", "/etc/deepwell.toml"]
