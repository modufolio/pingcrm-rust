#!/usr/bin/env bash
#
# PingCRM dev-loop shortcuts: build, test, check, fmt, clippy, watch.

_pingcrm_cargo() {
    (cd "$(get_project_root)" && cargo "$@")
}

# Build the entire workspace [dev,build]
cmd_dev_build() {
    _pingcrm_cargo build "$@"
}

# Fast type-check the workspace [dev,check]
cmd_dev_check() {
    _pingcrm_cargo check "$@"
}

# Run all tests (forwards extra args to cargo test) [dev,test]
cmd_dev_test() {
    _pingcrm_cargo test "$@"
}

# Run integration tests with single-threaded execution [dev,test]
cmd_dev_itest() {
    _pingcrm_cargo test -p app --test integration_tests -- --test-threads=1 "$@"
}

# Format the codebase with rustfmt [dev,fmt]
cmd_dev_fmt() {
    _pingcrm_cargo fmt --all
    success "Codebase formatted"
}

# Run clippy lints [dev,lint]
cmd_dev_clippy() {
    _pingcrm_cargo clippy --workspace --all-targets "$@"
}

# Clean build artifacts [dev,clean]
cmd_dev_clean() {
    info "Removing target/ ..."
    _pingcrm_cargo clean
    success "Build artifacts cleaned"
}

# Watch and rerun app-server (needs cargo-watch) [dev,watch]
cmd_dev_watch() {
    if ! command -v cargo-watch &>/dev/null; then
        error "cargo-watch not installed. Run: cargo install cargo-watch"
        exit 1
    fi
    (cd "$(get_project_root)" && cargo watch -x 'run -p app --bin app-server')
}
