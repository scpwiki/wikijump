#
# DEEPWELL build
#

FROM rust:alpine AS rust

# Install system dependencies
RUN apk update
RUN apk add --no-cache build-base libmagic-static libssh cmake file pkgconf

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

FROM alpine:latest

ENV LOCALIZATION_PATH="/opt/locales"

RUN apk update
RUN apk add --no-cache curl
COPY --from=rust /src/deepwell/target/release/deepwell /usr/local/bin/deepwell
COPY ./install/files/api/health-check.sh /bin/wikijump-health-check
COPY ./install/files/prod/deepwell.toml /etc/deepwell.toml
COPY ./locales/fluent /opt/locales/fluent

USER daemon
ENV PORT=2747
EXPOSE 2747
CMD ["/usr/local/bin/deepwell", "/etc/deepwell.toml"]
