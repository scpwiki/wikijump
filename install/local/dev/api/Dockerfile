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
# Final image
#

# We want alpine, but need glibc
FROM frolvlad/alpine-glibc:latest

RUN apk add --no-cache curl
COPY --from=rust /src/deepwell/target/release/deepwell /usr/local/bin/deepwell

USER daemon
CMD ["/usr/local/bin/deepwell"]
