#!/usr/bin/env bash

source "$HOME/.cargo/env"

cd ~

git clone mgit.bundle mgit

cd mgit

cargo build -p mgit-gui --release
