FROM rust:1.68-buster as builder

WORKDIR /app

# Install necessities
RUN apt-get update
RUN apt-get install libssl-dev pkg-config -y

COPY ./Cargo.lock ./
COPY ./Cargo.toml ./Cargo.toml

COPY /src ./src

ARG CARGO_NET_GIT_FETCH_WITH_CLI=true

RUN cargo build --release

FROM debian:bullseye-slim

RUN apt-get update

COPY --from=builder /app/target/release/rust-pepperbot /usr/local/bin/rust-pepperbot
