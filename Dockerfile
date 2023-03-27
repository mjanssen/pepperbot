FROM debian:bullseye-slim

RUN apt-get update

RUN apt-get install -y ca-certificates dumb-init
RUN update-ca-certificates --fresh

COPY message-queuing_$TARGETARCH /usr/local/bin/message-queuing
COPY bot-commands_$TARGETARCH /usr/local/bin/bot-commands
COPY bot-consumer_$TARGETARCH /usr/local/bin/bot-consumer
