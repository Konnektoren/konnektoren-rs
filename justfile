# Default recipe
default:
    @just --list

# Core package commands
core +COMMAND="build":
    @just -f konnektoren-core/justfile {{ COMMAND }}

# Platform package commands
platform +COMMAND="build":
    @just -f konnektoren-platform/justfile {{ COMMAND }}

# Tests package commands
tests +COMMAND="test":
    @just -f konnektoren-tests/justfile {{ COMMAND }}

# Add this to run BDD tests specifically
bdd:
    @just -f konnektoren-tests/justfile bdd

# Workspace-wide commands
build:
    cargo build --workspace

# Run tests
test:
    cargo test --workspace

# Run linter
lint:
    cargo clippy --workspace -- -D warnings

# Documentation
doc:
    cargo doc --workspace --no-deps

# Documentation with AsciiDoc
docs-build:
    cd docs && asciidoctor -r asciidoctor-diagram -o index.html -a imagesdir=images -a sourcedir=src README.adoc

# Documentation PDF
docs-pdf:
    cd docs && asciidoctor-pdf -r asciidoctor-diagram -o konnektoren-docs.pdf -a imagesdir=images -a sourcedir=src README.adoc

# Start Structurizr Lite using direct Docker commands
structurizr:
    docker run -it --rm -d \
      -p 8082:8080 \
      -v {{ justfile_directory() }}/docs:/usr/local/structurizr \
      --name structurizr-lite \
      structurizr/lite:latest
    @echo "Structurizr is starting at http://localhost:8082"
    #!/usr/bin/env sh
    if command -v xdg-open > /dev/null; then xdg-open http://localhost:8082; \
    elif command -v open > /dev/null; then open http://localhost:8082; \
    else echo "Could not detect the web browser opener"; fi

# Stop Structurizr
structurizr-down:
    docker stop structurizr-lite

# Coverage
coverage:
    cargo tarpaulin --workspace --all-features --ignore-tests --out Xml

# Generate SBOM
sbom:
    cargo cyclonedx

# Generate changelog
changelog:
    git-cliff -o CHANGELOG.md

# Clean build artifacts
clean:
    cargo clean

# Setup development environment
setup:
    rustup component add rustfmt
    rustup component add clippy
    cargo install cargo-tarpaulin
    cargo install cargo-cyclonedx
    cargo install git-cliff

# Pre-commit checks
pre-commit: lint test

tui +ARGS="":
    cargo run -p konnektoren-tui -- {{ ARGS }}

cli +ARGS="":
    cargo run -p konnektoren-tui --all-features -- {{ ARGS }}

tui-ssh +ARGS="":
    RUST_LOG=info cargo run -p konnektoren-tui --bin konnektoren-tui-ssh --features="ssh" -- {{ ARGS }}
