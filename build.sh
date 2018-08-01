#!/bin/sh
# Build script for Linux & macOS

cargo build --release
mv ./target/release/sbrx ./target/release/sbrx-$TRAVIS_OS_NAME
