# judger-rs

[![Tests](https://img.shields.io/github/workflow/status/OJ-lab/judger/Build)](https://github.com/OJ-lab/judger/actions/workflows/rust_build.yml)
[![Chat](https://img.shields.io/discord/916955582181822486)](https://discord.gg/bvdAt65v)

Judger is supposed to be a simple **sandbox service** which works for online-judge systems.

## System

judger-rs currently use `nix` to make necessary system invoke like `fork()`.

So you might need to check whether you are using the supported system from the main-page of [nix](https://github.com/nix-rust/nix).

**Briefly speaking, judger-rs is now supposing you are decided to run it on linux.**

We'll consider other platform, but in a lower priority.

## Develop in cloud

If you are not familiar with system stuff.
Developing judger-rs in local computer can be dangerous.

We setup `.gitpod.yml` for this project.
When you open this project in a **gitpod workplace**, you'll get everything necessary installed. And docker contained workplace can give you chances to make mistake.

And we pre-install strace for helping you analysing syscalls.

## Before run

judger-rs is based on `libseccomp`, so we need to have this library installed.

``` plain-text
sudo apt install libseccomp-dev
```

## Run by admin

`judge-core` build a sandbox environment by `seccomp`, so it's neccessary to run tests or examples by admin.
You need to install **Rust** in root user.

``` sh
su
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```
