# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- `ACCESSIBILITY.md` documenting input, color-fallback, and motion controls alongside known limitations.
- Automated weekly release PR workflow (`.github/workflows/weekly-release.yml`) and Release Cycle documentation in `CONTRIBUTING.md`.
- Cross-platform (`macos-latest`, `windows-latest`) build and Clippy check matrix in `rust-app.yml`.
- `SHA256SUMS.txt` release checksum artifact and `workflow_dispatch` / `workflow_call` triggers in `release.yml`.

### Changed
- Pinned all third-party GitHub Actions to immutable commit SHAs and added `concurrency:` cancellation groups to CI workflows.
- Grouped Dependabot minor/patch Cargo updates and GitHub Actions updates in `.github/dependabot.yml`.

## [0.1.1] - 2026-11-24

### Added
- Community governance files (`CONTRIBUTING.md`, `CODE_OF_CONDUCT.md`, `SECURITY.md`, `CHANGELOG.md`, `TRANSLATIONS.md`).
- Canonical single-file locale scaffold (`locales/en_us.yml`) and translation request issue template.
- Unified `Makefile` (`make check`, `make fmt`, `make lint`, `make test`).
- GitHub PR/Issue templates, Dependabot configuration, and tag-triggered `release.yml` workflow.

### Changed
- Updated `rand` to `0.10.4` and GitHub Actions workflow dependencies (`actions/checkout`, `softprops/action-gh-release`).
- Enforced `rustfmt` and strict Clippy/ShellCheck checks in CI.

## [0.1.0] - 2026-11-08

### Added
- Procedural retro arcade maze screensaver inspired by K.C. Munchkin! (1981).
- High-resolution 2×2 quadrant sub-pixel terminal renderer with automatic color capability detection (`truecolor`, `256`, `16`, `8`).
- K.C. Munchkin AI, 3 distinct colored ghosts (Red Bit, Green Middie, Yellow Byte), rotating central cage door, corner Super Pellets, moving pills, and side warp tunnels.
- Screen-rest intermission cutscene sequence (6+ alternating horizontal chase/flee sweeps on varying lines with scaled sprites).
- CLI options for color mode, FPS, speed, exit triggers (`--keyboard`, `--mouse`), starting sequence (`--intermission`, `--cutscene`, `--game`, `--sequence`), `--info`, `--help`, and `-v`/`--version`.
- Local install/uninstall scripts (`install.sh`, `uninstall.sh`) targeting `~/.local/bin/kcchase`.
- GitHub Actions CI workflows (`Build, Test & Lint` and `ShellCheck Lint`).
