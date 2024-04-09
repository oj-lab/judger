#!/bin/bash

if [ -x "$(command -v apt)" ]; then
    # if recently updated jump to installing dependencies
    if [ "$(find /var/cache/apt/pkgcache.bin -mmin -60)" ]; then
        echo 'Skipping apt update...'
    else
        echo 'Updating apt...'
        sudo apt update
    fi
    sudo apt install -y libseccomp-dev gcc curl pkg-config libssl-dev cmake gdb
fi

if [ ! -d "scripts/thirdparty" ]; then
    mkdir scripts/thirdparty
fi

echo 'Ensuring rustup is installed...'
if ! [ -x "$(command -v rustup)" ]; then
    echo 'rustup not found. Installing rustup...'
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs > scripts/thirdparty/rustup.sh
    chmod +x scripts/thirdparty/rustup.sh
    scripts/thirdparty/rustup.sh -y
fi

echo 'Ensuring rclone is installed...'
if ! [ -x "$(command -v rclone)" ]; then
    echo 'rclone not found. Installing rclone...'
    curl -sSf https://rclone.org/install.sh > scripts/thirdparty/rclone_install.sh
    # Rclone install script can take a fairly long time to download it's install package
    # So showing the download progress can be very helpful
    echo 'Adjusting rclone install script to show download progress...'
    sed -i 's/-OfsS/-OfS/g' scripts/thirdparty/rclone_install.sh
    chmod +x scripts/thirdparty/rclone_install.sh
    echo 'If it is taking too long to download the rclone install package, try manually install it'
    sudo scripts/thirdparty/rclone_install.sh
fi

echo 'Compiling built-in programs for judge-core testing...'
PWD=$(pwd)
cd judge-core/tests/data/built-in-programs && ./build.sh
cd "$PWD" || exit

echo 'Environment setup complete.'