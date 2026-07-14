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

protocol-test:
    jq empty tests/fixtures/herdr/pong.json tests/fixtures/herdr/session_snapshot.json tests/fixtures/herdr/error.json
    jq -c . tests/fixtures/herdr/events.jsonl >/dev/null
    cargo test --test environment --test protocol --test framing --test client --test subscription --test supervisor

domain-test:
    cargo test --test domain_types --test persona --test normalization --test guestbook --test reducer

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
