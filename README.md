# judger-rs

![GitHub Workflow Status](https://img.shields.io/github/actions/workflow/status/OJ-lab/judger/rust_build.yml)
![Discord](https://img.shields.io/discord/916955582181822486)

Judger is supposed to be a simple **sandbox service** which works for online-judge systems.

## System

judger-rs currently use `nix` to make necessary system invoke like `fork()`.

So you might need to check whether you are using the supported system from the main-page of [nix](https://github.com/nix-rust/nix).

**Briefly speaking, judger-rs is now supposing you are decided to run it on linux.**

We'll consider other platform, but in a lower priority.

## Develop in cloud

If you are not familiar with system stuff.
Developing judger-rs in local computer can be dangerous.

Github codespace is the currently the most perfered approach.
When first setup the cloud machine, run `ENV_SETUP.bash` to get essentials,
also install the recommended plugins provided in VSCode.

## Run by admin

`judge-core` build a sandbox environment by `seccomp`, so it's neccessary to run tests or examples by admin.
You need to install **Rust** in root user.

``` sh
su
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

## Contribute

We have a guide in judger's [WIKI](https://github.com/OJ-lab/judger/wiki/Contribution-Guide)
