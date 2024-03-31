# Judger Application

Judge server & cmd-line application built with `judger-core`.
the two mode are base on the same core logic, cmd-line mode is more convenient for debugging and testing.

In server mode, it fetches the judge tasks from [oj-lab-platform](https://github.com/OJ-lab/oj-lab-platform)
and reports the result back.

In cmd-line mode, by providing the necessary arguments, it can run the code in a sandboxed environment.

## How to use

Run `cargo run --bin judger` to get help.
