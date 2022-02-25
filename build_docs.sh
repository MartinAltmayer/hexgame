#!/usr/bin/env bash
set -eu
cd "$(dirname "$0")"

cargo readme > README.md
cargo doc --no-deps $@

