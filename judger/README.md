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

For an example usage of the lastest batch judge feature:

``` shell
cargo run --bin judger-cli -- batch-judge \
    -s test-collection/src/programs/read_and_write.cpp \
    -p test-collection/packages/icpc/hello_world \
    -l cpp \
    -t icpc \
    -r tmp/icpc
```

## Server

### How to run

`cargo run --bin judger-server -- --env-path ./judger/src/server/environment/.env.development`

### How to visit OpenAPI

visit `{HOST}/swagger-ui/`
