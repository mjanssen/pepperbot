FROM alpine:3.18

ARG TARGETARCH

RUN apk add --no-cache dumb-init

COPY --chmod=755 message-queuing_$TARGETARCH /usr/local/bin/message-queuing
COPY --chmod=755 bot-commands_$TARGETARCH /usr/local/bin/bot-commands
COPY --chmod=755 bot-consumer_$TARGETARCH /usr/local/bin/bot-consumer
COPY --chmod=755 webserver_$TARGETARCH /usr/local/bin/webserver

RUN echo -n `date '+v%Y.%m.%d.%H.%M'` > /etc/pepperbot_build
