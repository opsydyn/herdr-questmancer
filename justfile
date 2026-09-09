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
    cargo test --test domain_types --test persona --test normalization --test chronicle --test reducer

guild-test:
    cargo test --test app --test actions --test command --test runtime_loop --test input --test interaction --test scene_guild_hall --test scene_roster --test scene_interaction --test scene_overlays

delve-test:
    cargo test --test scene_pixel --test scene_delve --test scene_roster --test scene_stage --test scene_runtime --test scene_adapter --test runtime_loop --test interaction

persistence-test:
    cargo test --test config --test persisted_state --test atomic_state --test chronicle_persistence --test persistence_worker --test startup

# The count is positional: just property-test 4096
property-test cases="1024":
    PROPTEST_CASES={{cases}} cargo test --test property_domain --test persisted_state --test chronicle_capture

persistence-verify: verify persistence-test property-test release-check

release-check:
    cargo build --release
    git diff --check

verify: fmt-check lint test
    bash -n herdr/install.sh herdr/run.sh herdr/control.sh

release-verify: verify release-check

run view="guild":
    cargo run -- ui --view {{view}}

install-local:
    cargo build --release
    mkdir -p bin
    rm -f bin/questmancer
    cp target/release/questmancer bin/questmancer
    [ "$(uname)" = "Darwin" ] && codesign --force --sign - bin/questmancer || true

storybook:
    cargo run --features storybook --bin questmancer-storybook

storybook-test:
    cargo test --all-targets --features storybook

# Reclaim the build directory. `target/` reached 36 GB on disk here — 24 GB of
# dependency debug info and 10 GB of incremental state, which cargo grows
# across rebuilds and never prunes on its own. Run this when disk gets tight;
# the cost is one full rebuild.
clean-build:
    du -sh target 2>/dev/null || true
    cargo clean
    @echo "target/ reclaimed — the next build is a full one"

# What is actually taking the space, before deciding to delete anything.
disk:
    @du -sh . 2>/dev/null
    @du -sh target/debug/* 2>/dev/null | sort -rh | head -6
