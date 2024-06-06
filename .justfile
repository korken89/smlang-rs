# print options
default:
    @just --list --unsorted

# install cargo tools
init:
    cargo upgrade --incompatible
    cargo update

# check code
check:
    cargo check
    cargo fmt --all -- --check
    cargo clippy --all-targets --all-features

# fix clippy and fmt issues
fix:
    cargo clippy --allow-dirty --allow-staged --fix
    cargo fmt --all

# build project
build:
    cargo build --all-targets

# execute tests
test:
    cargo test

# execute benchmarks
bench:
    cargo bench
