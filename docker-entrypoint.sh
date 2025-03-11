#!/bin/bash

if [ -n "$ADMIN_TOKEN_FILE" ]; then
    if [ -f "$ADMIN_TOKEN_FILE" ]; then
        ADMIN_TOKEN="$(cat "$ADMIN_TOKEN_FILE")"
        export ADMIN_TOKEN
    fi
fi

export DISCORD_APPLICATION_ID=${DISCORD_APPLICATION_ID:-0}
export DISCORD_GUILD_ID=${DISCORD_GUILD_ID:-0}
export RUST_LOG=${RUST_LOG:-info,bot=trace}

envsubst < /bot.yaml.template > /etc/bot/bot.yaml

if [ $# -gt 0 ]; then
    exec /bot "$@"
else
    exec /bot --filename /etc/bot/bot.yaml start
fi
