#!/bin/bash

build=$(date +%FT%T%z)

parser-linux="target/x86_64-unknown-linux-musl/release/parser"
parser-windows="target/x86_64-pc-windows-gnu/release/parser.exe"

sender-linux="target/x86_64-unknown-linux-musl/release/sender"
sender-windows="target/x86_64-pc-windows-gnu/release/sender.exe"

if [ "$1" = "all" ]; then
  build=$build cargo build --release --all-features --all-targets --target=x86_64-pc-windows-gnu
  build=$build cargo build --release --all-features --all-targets --target=x86_64-unknown-linux-musl
elif [ "$1" = "offline" ]; then
  build=$build cargo build --release --all-features --all-targets --target=x86_64-pc-windows-gnu --offline
  build=$build cargo build --release --all-features --all-targets --target=x86_64-unknown-linux-musl --offline
elif [ "$1" = "check" ]; then
  build=$build cargo check --release --all-features --all-targets
else
  build=$build cargo build --release --all-features --all-targets --target=x86_64-unknown-linux-musl
fi

if [ -f $linux ]; then upx $linux; fi
if [ -f $windows ]; then upx $windows; fi
