# ictsc-kana

```
USAGE:
    bot --filename <CONFIG> <SUBCOMMAND>

OPTIONS:
    -f, --filename <CONFIG>
    -h, --help                 Print help information
    -V, --version              Print version information

SUBCOMMANDS:
    create-channels
    create-roles
    delete-channels
    delete-commands
    delete-roles
    help               Print this message or the help of the given subcommand(s)
    start
```

## Build

- needs docker

```
make build
```

## Run bot daemon

- needs docker

```
cp bot.yaml.example bot.yaml
make run
```

## Run oneshot command

```
cp bot.yaml.example bot.yaml
cargo run --release <subcommand>
```
