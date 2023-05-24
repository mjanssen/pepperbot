# Telegram Pepperbot

- https://t.me/pepperdeals_bot

## Development
- redis server: `docker compose up`
- bot commands: `cargo run --bin bot-commands` - enable bot slash commands
- bot consumer: `cargo run --bin bot-consumer` - consumer redis stream and send messages
- bot message queuing: `cargo run --bin message-queuing` - fetch rss details and put in stream
