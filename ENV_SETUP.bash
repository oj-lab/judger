#!/bin/bash

UBUNTU_FLAG="ubuntu"

CUR_SYSTEM=$(uname -a | tr '[:upper:]' '[:lower:]')

if [[ $CUR_SYSTEM =~ $UBUNTU_FLAG ]];then
    echo Current system is Ubuntu
    sudo apt update
    sudo apt install -y libseccomp-dev gcc
fi

curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh