#
# DEEPWELL build
#

FROM rust:alpine AS rust

# Install system dependencies
RUN apk update
RUN apk add --no-cache build-base libmagic-static file

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
# Framerail build
#

FROM node:19-alpine AS node

# Install pnpm
RUN npm install -g pnpm

# Copy sources
RUN mkdir /app
WORKDIR /app
COPY framerail/ ./
COPY assets ./src/assets

# Install app
RUN \
    pnpm install && \
    pnpm build && \
    pnpm prune --prod && \
    rm -rf src/ tests/

#
# Final image
#

FROM alpine:latest

ENV LOCALIZATION_PATH="/opt/locales"

RUN mkdir /app /opt/database
WORKDIR /app

RUN apk update
RUN apk add --no-cache curl nodejs libmagic-static file

COPY --from=rust /usr/local/cargo/bin/sqlx /usr/local/cargo/bin/sqlx
COPY --from=rust /src/deepwell/target/release/deepwell /usr/local/bin/deepwell
COPY --from=node /app /app/framerail

COPY ./install/files/api/health-check.sh /bin/deepwell-health-check
COPY ./install/files/dev/deepwell.toml /etc/deepwell.toml
COPY ./install/files/dev/deepwell-start /usr/local/bin/wikijump-deepwell-start
COPY ./locales/fluent /opt/locales/fluent
COPY ./deepwell/seeder /opt/database/seeder
COPY ./deepwell/migrations /opt/database/migrations

USER daemon
