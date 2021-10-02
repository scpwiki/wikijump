FROM rust:latest as rust

# Copy source
RUN mkdir /src
COPY ./ftml /src/ftml
WORKDIR /src/ftml

# Build ftml-ffi without logging
RUN cargo build \
        --release \
        --no-default-features \
        --features ffi,mathml
