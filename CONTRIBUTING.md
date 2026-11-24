# Contributing to kcchase

Thank you for your interest in contributing to **kcchase**!

## Local Setup

1. Install a recent Rust toolchain (Rust 2024 edition or newer) via [rustup](https://rustup.rs).
2. Install [`shellcheck`](https://www.shellcheck.net/) to lint the shell install scripts locally.
3. Fork and clone the repository:
   ```bash
   git clone https://github.com/HappyAmos/kcchase.git
   cd kcchase
   ```

## Running Checks Locally

Run the unified verification command before committing or opening a pull request. It executes the exact same checks run by GitHub Actions CI:

```bash
make check
```

This runs:
- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`
- `shellcheck install.sh uninstall.sh`

To auto-format Rust code:

```bash
make fmt
```

## Branching & Pull Request Workflow

Direct pushes to `main` are blocked by repository rules. All changes must go through a Pull Request:

1. Create a descriptive branch from an up-to-date `main`:
   - `feat/<short-description>` for new features
   - `fix/<short-description>` for bug fixes
   - `docs/<short-description>` for documentation updates
   - `chore/<short-description>` for maintenance, tooling, or CI changes
2. Make your changes and add/update unit tests as appropriate.
3. Update `CHANGELOG.md` under `## [Unreleased]` for user-visible changes.
4. Keep user-facing strings synchronized with [`locales/en_us.yml`](locales/en_us.yml) (see [TRANSLATIONS.md](TRANSLATIONS.md)).
5. Run `make check` and confirm zero warnings or failures.
6. Write clear, concise commit messages (Conventional Commits style recommended, e.g., `feat: ...`, `fix: ...`, `docs: ...`).
7. Push your branch and open a Pull Request against `main`, completing the PR checklist.

## Code of Conduct

All contributors are expected to uphold our [Code of Conduct](CODE_OF_CONDUCT.md).
