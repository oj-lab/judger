# Judger

[![Discord](https://img.shields.io/discord/916955582181822486?label=Discord&color=blue&logo=discord&logoColor=white)](https://discord.gg/vh8NCgdp8J)
![Codespace Supported](https://img.shields.io/badge/Codespace_Supported-000000?style=flat&logo=github)

Library & application which supports running code in a sandboxed environment.

## System Prerequisite

Judger currently use `nix` to make necessary system invoke like `fork()`.
So you might need to check whether you are using the supported system from the main-page of [nix](https://github.com/nix-rust/nix).

**Briefly speaking, judger-rs is now supposing you are decided to run it on linux.**

## Development

> 🌟 Accept VSCode extension recommandation for complete experience.

### Before you start

You may need to setup your environment before you start.
There is a setup script to help you quickly get ready.

> 🥰 You won't need to run this script if you are using GitHub Codespaces.

```sh
./scripts/env_setup.bash
```

### Debugging

Launch settings are already configured in `.vscode/launch.json`, try in the debug panel.

## Deeper Docs

- [judge-core README](judge-core/README.md)
