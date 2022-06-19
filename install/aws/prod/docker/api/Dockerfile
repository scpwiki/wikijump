#
# DEEPWELL build
#

FROM rust:latest AS rust

# Copy source
RUN mkdir /src
COPY ./deepwell /src/deepwell
WORKDIR /src/deepwell

# Install system dependencies
RUN apt update
RUN apt install libmagic-dev

# Cache rust dependencies
RUN cargo vendor

# Build deepwell server
RUN cargo build --release

#
# Final image
#

# We want alpine, but need glibc
FROM frolvlad/alpine-glibc:glibc-2.30

ENV LOCALIZATION_PATH="/opt/locales"

RUN apk add --no-cache curl libmagic
COPY --from=rust /src/deepwell/target/release/deepwell /usr/local/bin/deepwell
COPY ./locales/fluent /opt/locales/fluent
COPY ./install/files/api/health-check.sh /bin/wikijump-health-check
COPY ./install/files/prod/api_env /.env

USER daemon
CMD ["/usr/local/bin/deepwell"]
