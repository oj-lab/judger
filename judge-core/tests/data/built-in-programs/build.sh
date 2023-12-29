#!/bin/sh -e

mkdir -p build
cmake -B build --install-prefix "$(pwd)/build"
cmake --build build --parallel
cmake --install build
