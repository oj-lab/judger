# Docker Build Guide

⚠️ Do not use VSCode's `Build Image..` usage.

Run the following command under project root instead.

```sh
docker build --pull --rm -f "docker/judger-server.dockerfile" -t oj-lab/judger-server:latest .
```
