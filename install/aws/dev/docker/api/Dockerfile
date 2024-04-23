#
# DEEPWELL build
#

FROM rust:alpine AS rust

# Install system dependencies
RUN apk update
RUN apk add --no-cache libmagic

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

RUN apk add --no-cache curl libmagic
COPY --from=rust /src/deepwell/target/release/deepwell /usr/local/bin/deepwell
COPY ./install/files/api/health-check.sh /bin/wikijump-health-check
COPY ./locales/fluent /opt/locales/fluent
COPY ./install/files/dev/api_env /.env

USER daemon
CMD ["/usr/local/bin/deepwell"]
