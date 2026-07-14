default:
    @just --list

fmt:
    cargo fmt --all

fmt-check:
    cargo fmt --all --check

lint:
    cargo clippy --all-targets --all-features -- -D warnings

test:
    cargo test --all-targets --all-features
    bash tests/scripts.sh

verify: fmt-check lint test
    bash -n herdr/install.sh herdr/run.sh herdr/control.sh

run view="desk":
    cargo run -- ui --view {{view}}

install-local:
    cargo build --release
    mkdir -p bin
    rm -f bin/herdr-webmaster
    cp target/release/herdr-webmaster bin/herdr-webmaster
    [ "$(uname)" = "Darwin" ] && codesign --force --sign - bin/herdr-webmaster || true

