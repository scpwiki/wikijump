#
# DEEPWELL build
#

FROM rust:latest AS rust

# Copy source
RUN mkdir /src
COPY ./deepwell /src/deepwell
WORKDIR /src/deepwell

# Build deepwell server
RUN cargo build --release

#
# Localizations
#

# We have to duplicate this build process
# from php-fpm unfortunately :(

FROM python:3.9-slim AS locales

# Copy source
RUN mkdir /src
COPY ./locales /src/locales
WORKDIR /src/locales

# Install dependencies
RUN apt update && \
    apt install -y gettext
RUN pip3 install --upgrade -r requirements.txt

# Build localization files
RUN python3 -m messages .

#
# Final image
#

# We want alpine, but need glibc
FROM frolvlad/alpine-glibc:latest

ENV LOCALIZATION_PATH="/opt/locales"

RUN apk add --no-cache curl
COPY --from=rust /src/deepwell/target/release/deepwell /usr/local/bin/deepwell
COPY --from=locales /src/locales/out/ /opt/locales

USER daemon
CMD ["/usr/local/bin/deepwell"]
