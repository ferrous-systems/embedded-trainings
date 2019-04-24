#!/bin/bash

curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- --default-toolchain stable -y

source $HOME/.cargo/env

mkdir -p ./netlify/beginner

cd beginner/templates/
cargo doc --all --target x86_64-unknown-linux-gnu

mv ./target/x86_64-unknown-linux-gnu/doc/ ../../netlify/beginner
