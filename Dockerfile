FROM debian:bullseye-slim

ARG TARGETARCH

RUN apt-get update

RUN apt-get install -y ca-certificates dumb-init
RUN update-ca-certificates --fresh

COPY message-queuing_$TARGETARCH /usr/local/bin/message-queuing
COPY bot-commands_$TARGETARCH /usr/local/bin/bot-commands
COPY bot-consumer_$TARGETARCH /usr/local/bin/bot-consumer
COPY webserver_$TARGETARCH /usr/local/bin/webserver

RUN chmod 755 /usr/local/bin/message-queuing \
    /usr/local/bin/bot-commands \
    /usr/local/bin/bot-consumer \
    /usr/local/bin/webserver

RUN echo -n `date '+v%Y.%m.%d.%H.%M'` > /etc/pepperbot_build
