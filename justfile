target_dir := "target"

#----------
# Building
#----------

build: check-formatting test test-all build-simulator check-readme generate-docs

# Build the simulator
build-simulator:
    cargo build --release --no-default-features

# Run cargo test in release mode
test:
    cargo test --release

# Run cargo test in release mode with all features enabled
test-all:
    cargo test --release --all-features

# Check the formatting
check-formatting:
    cargo fmt --all -- --check

#------
# Docs
#------

# Generates the docs
generate-docs:
    cargo clean --doc
    cargo doc --all-features

# Runs cargo-deadlinks on the docs
check-links: generate-docs
    cargo deadlinks

#----------------------
# README.md generation
# ---------------------

# Generate README.md for a single crate
generate-readme: (_build-readme)
    cp {{target_dir}}/README.md README.md

# Check README.md for a single crate
@check-readme: (_build-readme)
    diff -q {{target_dir}}/README.md README.md || ( \
        echo -e "\033[1;31mError:\033[0m README.md needs to be regenerated."; \
        echo -e "       Run 'just generate-readme' to regenerate.\n"; \
        exit 1 \
    )

# Builds README.md for a single crate
_build-readme:
    #!/usr/bin/env bash
    set -e -o pipefail
    mkdir -p {{target_dir}}/readme
    echo "Building README.md"
    cargo readme | sed -E -f filter_readme.sed > {{target_dir}}/README.md
