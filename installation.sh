#!/bin/bash

# Install Rust via rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

# Ensure the script runs in the current shell session
source $HOME/.cargo/env

# Update package lists
sudo apt-get update

# Install protobuf-compiler
# sudo apt-get -y install protobuf-compiler>=23.4

sudo apt-get remove protobuf-compiler
 
wget https://github.com/protocolbuffers/protobuf/releases/download/v23.4/protoc-23.4-linux-x86_64.zip

 
sudo apt-get install unzip
unzip protoc-23.4-linux-x86_64.zip -d protoc_install
sudo cp protoc_install/bin/protoc /usr/local/bin/
sudo cp -r protoc_install/include/* /usr/local/include/

 
# protoc --version
