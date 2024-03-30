# Judger

![GitHub Workflow Status](https://img.shields.io/github/actions/workflow/status/OJ-lab/judger/rust-check.yml?logo=github&label=Tests)
[![Discord](https://img.shields.io/discord/916955582181822486?label=Discord&color=blue&logo=discord&logoColor=white)](https://discord.gg/vh8NCgdp8J)
![Codespace Supported](https://img.shields.io/badge/Codespace_Supported-000000?style=flat&logo=github)

Judger is supposed to be a simple **sandbox service** which works for online-judge systems.

## System

judger-rs currently use `nix` to make necessary system invoke like `fork()`. 
So you might need to check whether you are using the supported system from the main-page of [nix](https://github.com/nix-rust/nix).

**Briefly speaking, judger-rs is now supposing you are decided to run it on linux.**
We'll consider other platform, but in a lower priority.

## Contribute

We have a guide in judger's [WIKI](https://github.com/OJ-lab/judger/wiki/Contribution-Guide)

## Deeper Docs

- [judge-core README](judge-core/README.md)
