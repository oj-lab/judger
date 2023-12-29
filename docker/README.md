# Docker Build Guide

⚠️ Do not use VSCode's `Build Image..` usage.

Run the following command under project root instead.

```sh
docker build --pull --rm -f "docker/judge-server.dockerfile" -t oj-lab/judge-server:latest .
```
