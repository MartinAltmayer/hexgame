#!/usr/bin/env bash
set -eu
cd "$(dirname "$0")"

# Note: This script recquires cargo-readme
# Unfortunately, this crate cannot be installed as dev dependency.
# And even if it could be installed, it wouldn't work:
# https://github.com/rust-lang/cargo/issues/2267

cargo readme > README.md
cargo doc --no-deps $@

