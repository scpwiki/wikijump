#
# DEEPWELL build
#

FROM rust:alpine AS rust

# Install system dependencies
RUN apk update
RUN apk add --no-cache build-base libmagic file pkgconf

# Install helpers
RUN cargo install sqlx-cli --no-default-features --features rustls,postgres

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

# Install system dependencies
RUN mkdir /opt/database
RUN apk update
RUN apk add --no-cache curl libmagic-static file postgresql-client

# Install files
COPY --from=rust /usr/local/cargo/bin/sqlx /usr/local/cargo/bin/sqlx
COPY --from=rust /src/deepwell/target/release/deepwell /usr/local/bin/deepwell
COPY --from=rust /src/deepwell/migrations /opt/database/migrations
COPY --from=rust /src/deepwell/seeder /opt/database/seeder
COPY install/files/dev/deepwell.toml /etc/deepwell.toml
COPY install/files/dev/deepwell-start /usr/local/bin/wikijump-deepwell-start
COPY install/files/api/health-check.sh /usr/local/bin/wikijump-health-check
COPY locales/fluent /opt/locales/fluent

USER daemon
EXPOSE 2747
CMD ["/usr/local/bin/wikijump-deepwell-start"]
