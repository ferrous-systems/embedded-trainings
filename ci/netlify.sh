#!/bin/bash

curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- --default-toolchain stable -y

printenv

mkdir -p ./netlify

cd beginner/templates/segment-1
cargo doc --target x86_64-unknown-linux-gnu
cd ../segment-3
cargo doc --target x86_64-unknown-linux-gnu
cd ../segment-4
cargo doc --target x86_64-unknown-linux-gnu
cd ..

mv ./segment-1/target/x86_64-unknown-linux-gnu/doc ../../netlify/segment-1-docs
mv ./segment-3/target/x86_64-unknown-linux-gnu/doc ../../netlify/segment-3-docs
mv ./segment-4/target/x86_64-unknown-linux-gnu/doc ../../netlify/segment-4-docs
