# Judger Application

Judge server & cmd-line application built with `judger-core`.
the two mode are base on the same core logic, cmd-line mode is more convenient for debugging and testing.

In server mode, it fetches the judge tasks from [platform](https://github.com/oj-lab/platform)
and reports the result back.

In cmd-line mode, by providing the necessary arguments, it can run the code in a sandboxed environment.

## Development

Use VSCode Run/Debug configuration to run the application.

## How to use

Run `cargo run --bin judger` to get help.
