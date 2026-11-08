# Development Guidelines for kcchase

## Terminal Canvas & Sub-Pixel Rendering
- **Cursor Positioning**: Always position rows explicitly with ANSI escape code `\x1b[{row};1H` rather than relying on `\n` or terminal line wrapping.
- **DECAWM Autowrap**: Always disable autowrap during canvas rendering using `\x1b[?7l` and restore with `\x1b[?7h` in RAII guard teardown to prevent double-spaced alternate empty rows.
- **Screensaver Anti-Burn-In**: Maintain zero static screen elements. Regenerate maze layouts, neon colors, and entity configurations upon each catch or round completion.

## Procedural Maze & Entity AI Guidelines
- **100% Graph Connectivity**: Every procedural maze generation must guarantee a connected spanning graph (Union-Find) so all path tiles, K.C., ghosts, and corner pellets are mutually reachable.
- **Corner Super Pellets**: Always place 4 Super Pellets in the 4 corner quadrants. Ensure K.C. can re-energize if touching a remaining corner pellet during chase or flee states.
- **Warp Tunnels**: Side warps must use dedicated off-screen target coordinates and a `warping` state flag to prevent entities from clipping through interior maze walls during coordinate wrapping.

## Installation & Packaging
- Maintain `install.sh` and `uninstall.sh` targeting `~/.local/bin/kcchase`.
- Check for `cargo` prerequisite before building, providing instructions to install Rust via `rustup` if missing.
