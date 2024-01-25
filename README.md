# judger-rs

![GitHub Workflow Status](https://img.shields.io/github/actions/workflow/status/OJ-lab/judger/rust-check.yml)
![Discord](https://img.shields.io/discord/916955582181822486?label=discord&color=blue)

Judger is supposed to be a simple **sandbox service** which works for online-judge systems.

## System

judger-rs currently use `nix` to make necessary system invoke like `fork()`.

So you might need to check whether you are using the supported system from the main-page of [nix](https://github.com/nix-rust/nix).

**Briefly speaking, judger-rs is now supposing you are decided to run it on linux.**

We'll consider other platform, but in a lower priority.

## Develop in cloud

If you are not familiar with system stuff.
Developing judger-rs in local computer can be dangerous.

Github codespace is the currently the most perfered approach,
it will setup all the needed environment for you.

## Contribute

We have a guide in judger's [WIKI](https://github.com/OJ-lab/judger/wiki/Contribution-Guide)

## Deeper Docs

- [judge-core README](judge-core/README.md)
