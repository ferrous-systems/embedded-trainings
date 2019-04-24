#!/bin/bash

curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- --default-toolchain stable -y

export PATH=$PATH:$HOME/.cargo/bin

mkdir -p ./netlify/beginner

cd beginner/templates/
cargo doc --all --target x86_64-unknown-linux-gnu

mv ./target/x86_64-unknown-linux-gnu/doc/ ../../netlify/beginner
