# Judger

## Cli

This will be the easiest and closest way to try out the basic feature of Judger.

### Develop Usage

To run from the source code, try:

``` shell
cargo run --bin judger-cli -- [COMMAND]
```

``` shell
cargo run --bin judger-cli -- batch-judge --help
```

## Server

### How to run

`cargo run --bin judger-server -- --env-path ./judger/src/server/environment/.env.development`

### How to visit OpenAPI

visit `{HOST}/swagger-ui/`
