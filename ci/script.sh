#!/usr/bin/env bash

set -euxo pipefail

main() {
    cargo test
    cargo check
}

main
