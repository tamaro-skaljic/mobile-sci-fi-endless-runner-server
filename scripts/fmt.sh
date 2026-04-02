#/bin/sh

cargo fmt --all -- --check

cargo clippy --fix --allow-dirty --all-features
