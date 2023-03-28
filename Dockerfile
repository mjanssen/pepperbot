FROM debian:bullseye-slim

ARG TARGETARCH

RUN apt-get update

RUN apt-get install -y ca-certificates dumb-init
RUN update-ca-certificates --fresh

COPY message-queuing_$TARGETARCH /usr/local/bin/message-queuing
COPY bot-commands_$TARGETARCH /usr/local/bin/bot-commands
COPY bot-consumer_$TARGETARCH /usr/local/bin/bot-consumer

RUN chmod 755 /usr/local/bin/message-queuing \
    /usr/local/bin/bot-commands \
    /usr/local/bin/bot-consumer
