FROM rust:1.68-buster as builder

WORKDIR /app

# Install necessities
RUN apt-get update
RUN apt-get install libssl-dev pkg-config -y

COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

ARG CARGO_NET_GIT_FETCH_WITH_CLI=true

RUN mkdir -p src && echo "fn main() {}" > src/main.rs && cargo build --release && rm -rf ./src/ target/release/deps/rust-pepperbot* target/release/rust-pepperbot*

COPY src/ ./src/

RUN cargo build --release

FROM debian:bullseye-slim

RUN apt-get update

RUN apt-get install -y ca-certificates
RUN update-ca-certificates --fresh

COPY --from=builder /app/target/release/rust-pepperbot /usr/local/bin/rust-pepperbot
