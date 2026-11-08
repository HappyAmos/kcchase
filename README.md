# kcchase

A high-resolution retro terminal screensaver inspired by the classic Magnavox Odyssey² / Philips Videopac game **K.C. Munchkin!** (1981).

## Features

- **Anti-Burn-In Procedural Arcade Mazes**:
  - Replaces all static borders and corners with **randomized Pac-Man / Odyssey² style mazes** featuring symmetrical corridors, wall blocks, T-junctions, and edge warp tunnels.
  - Characters navigate strictly through the open corridors and turns using BFS pathfinding.
  - **Dynamic Maze Refresh**: As soon as one character catches the other (K.C. chomps the vulnerable ghost, or the hunting ghost catches K.C.), the maze explodes in celebratory retro arcade particles and **immediately regenerates a completely new random maze pattern** with new wall layouts, glowing neon colors, character sizes, and pellet routes. This guarantees no static pixels linger on screen.
- **Iconic Odyssey² Animation Cycle**:
  1. K.C. Munchkin navigates corridors toward a pulsing **Super Pellet**, chomping normal pellets along the way.
  2. Chomping the Super Pellet triggers an expanding shockwave; K.C. gains an energized aura while the Muncher ghost turns frightened blue and flees through maze corners.
  3. K.C. aggressively chases the ghost through the corridors.
  4. The power effect fades with the ghost flashing warning colors.
  5. The ghost reverts to its normal color, turns around, and actively hunts K.C. through the maze.
  6. K.C. flees in terror with wide eyes and flying sweat drops.
  7. When either character catches the other, the map refreshes into a fresh random maze!
- **High-Resolution ANSI Block Graphics**:
  - Uses Unicode quadrant (`▘▝▖▗▌▐▞▚▛▜▙▟█`) and half-block (`▀▄`) sub-pixels to render 2×2 virtual pixels per terminal character cell.
  - Smooth antialiased procedural sprites for K.C.'s horns, chomping mouth, animated eyes, and the ghost's wavy skirt ripples.
- **Full Terminal Screen Fitting & Dynamic Resizing**:
  - Automatically adjusts to the full terminal screen size.
  - Handles terminal resize events (`SIGWINCH`) smoothly without tearing or artifacts.
- **Color Capability Detection**:
  - Evaluates terminal capabilities (via `COLORTERM` and `TERM`).
  - Supports **Truecolor** (24-bit RGB), **256 Color** (8-bit), **16 Color** (4-bit ANSI), and **8 Color** (3-bit ANSI) with weighted perceptual color distance mapping.
  - Can be manually overridden with `--color`.
- **Pure Screensaver Exit**:
  - Exits immediately on **any keyboard press** or **mouse movement**.
  - RAII guard and panic handler ensure terminal state (raw mode, cursor visibility, mouse capture, alternate screen) is always cleanly restored upon exit.

## Installation & Building

Requires a Rust toolchain (Rust 2024 edition or newer).

```bash
# Clone the repository
git clone https://github.com/AndrewAlbrightTX/kcchase.git
cd kcchase

# Build the release binary
cargo build --release
```

The optimized executable will be located at `target/release/kcchase`.

## Usage

Run directly as a screensaver:

```bash
./target/release/kcchase
```

### Options

```
kcchase - K.C. Munchkin Terminal Screensaver

USAGE:
    kcchase [OPTIONS]

OPTIONS:
    -h, --help               Print this help message and exit
    -i, --info               Display detected terminal color capabilities and exit
    -c, --color <MODE>       Select color mode: truecolor, 256, 16, 8 (default: auto)
    --fps <NUMBER>           Set target frame rate (default: 60, range: 15-120)
    --speed <FLOAT>          Set animation speed multiplier (default: 1.0)

EXIT:
    Press any key or move the mouse to exit the screensaver.
```

### Inspecting Terminal Capabilities

```bash
./target/release/kcchase --info
```

Output:
```
Terminal Color Capability Detection:
  Active Mode: Truecolor (24-bit)
  COLORTERM:   Some("truecolor")
  TERM:        Some("xterm-256color")

Available Modes: truecolor (24-bit), 256 (8-bit), 16 (4-bit), 8 (3-bit)
```

## License

MIT
