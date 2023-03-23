FROM rust:1.68-buster as builder

WORKDIR /app

# Install necessities
RUN apt-get update
RUN apt-get install libssl-dev pkg-config -y

COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

ARG CARGO_NET_GIT_FETCH_WITH_CLI=true

RUN mkdir -p src \
    && echo "fn main() {}" > src/message_queuing.rs \
    && echo "fn main() {}" > src/bot_commands.rs \
    && echo "fn main() {}" > src/bot_consumer.rs \
    && cargo build --release \
    && rm -rf ./src/ target/release/deps/message_queuing* target/release/message_queuing* \
    && rm -rf ./src/ target/release/deps/bot_commands* target/release/bot_commands* \
    && rm -rf ./src/ target/release/deps/bot_consumer* target/release/bot_consumer*

COPY src/ ./src/

RUN cargo build --bins --release

FROM debian:bullseye-slim

RUN apt-get update

RUN apt-get install -y ca-certificates dumb-init
RUN update-ca-certificates --fresh

COPY --from=builder /app/target/release/message-queuing /usr/local/bin/message-queuing
COPY --from=builder /app/target/release/bot-commands /usr/local/bin/bot-commands
COPY --from=builder /app/target/release/bot-consumer /usr/local/bin/bot-consumer

