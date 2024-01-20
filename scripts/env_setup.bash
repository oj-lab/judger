#!/bin/bash

if [ -x "$(command -v apt)" ]; then
    sudo apt update
    sudo apt install -y libseccomp-dev gcc curl pkg-config libssl-dev
fi

if [ ! -d "scripts/thirdparty" ]; then
    mkdir scripts/thirdparty
fi

echo 'Ensuring rustup is installed...'
if ! [ -x "$(command -v rustup)" ]; then
    echo 'rustup not found. Installing rustup...'
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs > scripts/thirdparty/rustup.sh
    chmod +x scripts/thirdparty/rustup.sh
    sudo scripts/thirdparty/rustup.sh -y
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

echo 'Environment setup complete.'