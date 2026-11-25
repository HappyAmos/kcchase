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

## Release Cycle

`kcchase` uses **trunk-based development** on `main`:

- `main` is the live, always-tested development branch.
- Releases follow [Semantic Versioning](https://semver.org/) (`vMAJOR.MINOR.PATCH`) and are documented in [`CHANGELOG.md`](CHANGELOG.md) in [Keep a Changelog](https://keepachangelog.com/) format.
- Already-published `v*.*.*` tags are immutable and never moved.
- An automated weekly release workflow (`.github/workflows/weekly-release.yml`) runs every Friday evening (and can be triggered manually via `workflow_dispatch`):
  1. Checks for new user-facing commits (`feat`, `fix`, `perf`, `refactor`) since the latest tag and skips quietly if none exist.
  2. Opens or updates a `chore(release): prepare vX.Y.Z` Pull Request on a `release/vX.Y.Z` branch that bumps `Cargo.toml`/`Cargo.lock` and rolls `CHANGELOG.md` entries into the new version heading.
  3. Once reviewed and squash-merged into `main`, automatically creates and pushes the annotated `vX.Y.Z` tag and invokes `.github/workflows/release.yml` to build, verify, checksum (`SHA256SUMS.txt`), and publish the GitHub Release.

## Code of Conduct

All contributors are expected to uphold our [Code of Conduct](CODE_OF_CONDUCT.md).
