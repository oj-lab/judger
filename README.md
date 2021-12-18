# judger-rs

[![Tests](https://img.shields.io/github/workflow/status/slhmy/judger-rs/Build)](https://github.com/slhmy/judger-rs/actions/workflows/rust_build.yml)

Judger is supposed to be a simple **sandbox service** which works for online-judge systems.

## Before run
judger-rs is based on `libseccomp`, so we need to have this library installed.

```
sudo apt install libseccomp-dev
```

## Run by admin

`judge-core` build a sandbox environment by `seccomp`, so it's neccessary to run tests or examples by admin.
You need to install **Rust** in root user.
``` sh
su
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```