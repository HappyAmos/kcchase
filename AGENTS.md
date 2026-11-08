# Development Guidelines for kcchase

## Terminal Canvas & Sub-Pixel Rendering
- **Cursor Positioning**: Always position rows explicitly with ANSI escape code `\x1b[{row};1H` rather than relying on `\n` or terminal line wrapping.
- **DECAWM Autowrap**: Always disable autowrap during canvas rendering using `\x1b[?7l` and restore with `\x1b[?7h` in RAII guard teardown to prevent double-spaced alternate empty rows.
- **Screensaver Anti-Burn-In**: Maintain zero static screen elements. Regenerate maze layouts, neon colors, and entity configurations upon each catch or round completion.

## Procedural Maze & Entity AI Guidelines
- **100% Graph Connectivity**: Every procedural maze generation must guarantee a connected spanning graph (Union-Find) so all path tiles, K.C., ghosts, and corner pellets are mutually reachable.
- **Corner Super Pellets**: Always place 4 Super Pellets in the 4 corner quadrants. Ensure K.C. can re-energize if touching a remaining corner pellet during chase or flee states.
- **Warp Tunnels**: Side warps must use dedicated off-screen target coordinates and a `warping` state flag to prevent entities from clipping through interior maze walls during coordinate wrapping.
- **K.C. Munchkin AI Priorities**: K.C.'s primary objective is always collecting all pills to advance the round. Chasing vulnerable ghosts is strictly a secondary, opportunistic bonus (only pursued when within immediate range <= 3 tiles or closer than pills).
- **Pursuit & Anti-Stall Forward Momentum**: When pursuing a moving entity (ghost or pill), if the chaser reaches the target's current tile, targeting must advance to the target's destination tile (`target_col`, `target_row`) or maintain the target's direction. Never allow BFS zero-distance returns (`start == goal -> None`) to freeze movement at tile centers.
- **Collision Catch Thresholds**: Entity catch thresholds must account for character radius overlap (`(character_radius * 2.2).max(cell_w.min(cell_h) * 0.95)` or `same_tile`).

## Animation Phases & State Invariants
- **Victory & Defeat Phase Freezes**: During `Phase::GhostCatchKc` and `Phase::KcCheer`, entity position updates and collision checks must be bypassed at the start of `update()` so phase timers decrement without being repeatedly reset by continuous entity contact.
- **Ghost State During Victory**: Ghosts must pause at their current coordinates during `Phase::KcCheer` while their wavy skirts and eyes continue animating in place.
- **Round Advance & Scoring**: When all pills are cleared, advance to a fresh procedural maze pass with 12 new moving pills and carry the accumulated score forward. On K.C. defeat, reset score to 0.
- **In-Memory High Score**: High score is tracked strictly in memory for the HUD display; never write the high score to disk or filesystem.

## Visual Palette & Entity Colors
- **K.C. Munchkin**: Always render K.C. in vibrant arcade cyan (`RGB(0, 230, 255)`), with cyan limbs and white gloves during victory cheering.
- **Ghost Colors**: The three ghosts must each have distinct base colors: Red Bit (`RGB(255, 30, 30)`), Green Middie (`RGB(40, 220, 80)`), and Yellow Byte (`RGB(255, 220, 30)`).

## Installation & Packaging
- Maintain `install.sh` and `uninstall.sh` targeting `~/.local/bin/kcchase`.
- Check for `cargo` prerequisite before building, providing instructions to install Rust via `rustup` if missing.
