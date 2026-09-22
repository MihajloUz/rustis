FROM rust:1.97-alpine AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN apk add --no-cache \
    musl-dev \
    openssl-dev \
    openssl-libs-static \
    pkgconfig


RUN cargo build --release

FROM alpine:latest
COPY --from=builder /app/target/release/rustis /usr/local/bin/rustis
WORKDIR /app
CMD ["rustis"]

