# Judge Cli

This will be the easiest and closest way to try out the basic feature of Judger.

## Develop Usage

To run from the source code, try:

``` shell
cargo run --bin judge-cli -- [COMMAND]
```

``` shell
cargo run --bin judge-cli -- batch-judge --help
```

For an example usage of the lastest batch judge feature:

``` shell
cargo run --bin judge-cli -- batch-judge \
    -s test-collection/src/programs/read_and_write.cpp \
    -p test-collection/packages/icpc/hello_world \
    -l cpp \
    -t icpc \
    -r tmp/icpc
```