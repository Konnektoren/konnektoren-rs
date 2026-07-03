# konnektoren-rs

Rust implementation of Konnektoren

[![codecov](https://codecov.io/gh/konnektoren/konnektoren-rs/branch/main/graph/badge.svg)](https://codecov.io/gh/konnektoren/konnektoren-rs)

## Overview

This repository is a Rust workspace for the Konnektoren project, providing grammar learning and interactive exercises. The workspace consists of several packages:

- **konnektoren-core**:
  The main library containing core domain logic, challenge types, game state, achievements, and persistence.

- **konnektoren-platform**:
  Platform-specific utilities, internationalization (i18n), domain configuration, and asset management.

- **konnektoren-page**:
  Ratzilla-powered WebAssembly landing page for GitHub Pages, with a terminal-style index for project links.

- **konnektoren-tests**:
  BDD and integration tests for the project, using Cucumber for feature-driven testing.

- **konnektoren-tui**:
  Terminal User Interface (TUI) application for interacting with Konnektoren challenges and games.

## Build

```bash
just build
```

## Documentation

Documentation is available online:

* [Web TUI index](https://konnektoren.github.io/konnektoren-rs/)
* [Project docs](https://konnektoren.github.io/konnektoren-rs/docs/)
* [Cargo API docs](https://konnektoren.github.io/konnektoren-rs/doc/)
* [konnektoren_core API](https://konnektoren.github.io/konnektoren-rs/doc/konnektoren_core/)
* [GitHub repository](https://github.com/konnektoren/konnektoren-rs)

## Run

Open the Web TUI index at https://konnektoren.github.io/konnektoren-rs/.

Serve the Web TUI locally:

```bash
just page-serve
```

Build the Web TUI locally:

```bash
just page-build
```

Run the terminal TUI:

```bash
just tui
```

## Tools (`konnektoren-platform`)

The platform crate exposes development tools behind the `tools` feature flag.

Declare the feature in the consuming crate's `Cargo.toml` and register the
integration test with `required-features` so cargo errors clearly when the
feature is missing instead of silently running zero tests:

```toml
[dev-dependencies]
konnektoren-platform = { path = "../konnektoren-platform", features = ["tools"] }

[[test]]
name = "challenge_i18n"
required-features = ["tools"]
```

Run the integration tests with:

```bash
cargo test -p konnektoren-platform --features tools --test challenge_i18n
```

### `I18nChecker` — source-code translation coverage

Scans Rust source files for `i18n.t("key")` / `t_with_lang("key")` calls and
reports which keys are missing from each language's translation file.

```rust
use konnektoren_platform::i18n::{CombinedTranslationAsset, I18nAssets, I18nConfig};
use konnektoren_platform::tools::I18nChecker;

let config = I18nConfig::with_assets(CombinedTranslationAsset::<I18nAssets>::new("i18n.yml"));
let report = I18nChecker::new(config)
    .exclude_tests()
    .check_directory("src/");

if report.has_errors {
    eprintln!("{}", report.as_report().unwrap());
}
```

Use `report.missing_as_yaml()` to generate a ready-to-paste YAML snippet for missing keys.

### `ChallengeI18nChecker` — challenge translation coverage

Verifies that every `name` and `description` of the challenges in a [`GamePath`]
is translated for all builtin languages (`en`, `de`, `es`, `ar`, `zh`, `uk`, `pl`, `tr`, `vi`).
Keys are the strings from the path YAML; values are the per-language translations.

Translation files follow the naming convention `<path_id>_<lang>.json` (e.g.
`konnektoren_path_de.json`) and live in `assets/challenges/i18n/`.

Define an embedded asset struct and write a test:

```rust
use konnektoren_platform::i18n::JsonTranslationAsset;
use konnektoren_platform::tools::ChallengeI18nChecker;
use konnektoren_core::game::GamePath;

#[derive(rust_embed::RustEmbed)]
#[folder = "assets/challenges/i18n/"]
struct ChallengeI18nAssets;

#[test]
fn all_langs_translated() {
    let asset = JsonTranslationAsset::<ChallengeI18nAssets>::new();
    let report = ChallengeI18nChecker::new(&asset).check(&GamePath::default());
    report.assert_complete(); // panics and lists every missing key per language
}
```

## Justfile Commands

This project uses a `Justfile` for simplified command execution. Here are some commonly used commands:

*   **`just build`**: Builds the entire workspace.
*   **`just test`**: Runs all tests in the workspace.
*   **`just lint`**: Runs the linter on the entire workspace.
*   **`just doc`**: Generates documentation for the entire workspace.
*   **`just page-build`**: Builds the Ratzilla Web TUI index.
*   **`just page-serve`**: Serves the Ratzilla Web TUI index locally.
*   **`just coverage`**: Generates code coverage reports for the entire workspace.
*   **`just sbom`**: Generates SBOM.
*   **`just changelog`**: Generate changelog.
*   **`just clean`**: Cleans build artifacts.
*   **`just setup`**: Sets up the development environment.
*   **`just pre-commit`**: Runs pre-commit checks (lint and test).
*   **`just core build`**: Builds the konnektoren core package.
*   **`just platform build`**: Builds the konnektoren platform package.
*   **`just tests test`**: Run tests.
*   **`just bdd`**: Run BDD Tests

You can list all available commands by running:

```bash
just --list
```
