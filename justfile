target_dir := "target"
doc_dir := "doc"
doc_assets_dir := doc_dir + "/assets"

#----------
# Building
#----------

build: check-formatting test test-all build-simulator check-drawing-examples check-readme check-links

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

#----------
# Examples
#----------

# Generates the drawing examples screenshots and markdown file
generate-drawing-examples:
    #!/usr/bin/env bash
    set -ex -o pipefail
    mkdir -p {{doc_assets_dir}}
    cargo run --bin generate-drawing-examples | \
        rustfmt +nightly --config-path rustfmt.examples.toml | \
        sed -E -e 's@//! ?@@g' -e '/^# .*/d' -e '/pub mod dummy \{\}/d' - \
        > {{doc_dir}}/drawing-examples.md

# Checks if drawing examples are up to date
check-drawing-examples: generate-drawing-examples
    git diff --quiet doc/ || ( \
        echo "doc/ folder is not up to date" \
        echo "Try running 'just generate-drawing-examles'." \
        echo "If any images have changed, run just generate-drawing-examples-montage' to update the collage image too" \
    )

# Generate a collage of all drawing example screenshots
generate-drawing-examples-montage:
    # `imagemagick` must be installed for this to work.
    montage \
        {{doc_assets_dir}}/draw*.png \
        {{doc_assets_dir}}/display*.png \
        -tile 6x2 -background none -geometry 128x128+4+4 miff:- | \
    convert - -trim {{doc_assets_dir}}/all_drawing_ops.png
