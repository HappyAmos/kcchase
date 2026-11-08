# kcchase

A high-resolution retro terminal screensaver inspired by the classic Magnavox Odyssey² / Philips Videopac game **K.C. Munchkin!** (1981).

## Features

- **Anti-Burn-In Procedural Arcade Mazes**:
  - Replaces all static borders and corners with **randomized Pac-Man / Odyssey² style mazes** featuring symmetrical corridors, wall blocks, T-junctions, and edge warp tunnels.
  - Generates corridors with guaranteed 100% graph connectivity (using Kruskal's spanning tree with Union-Find and reachable flood-filling). K.C., the ghost, and the super pellet are mathematically guaranteed to be in the same mutually reachable maze component.
  - Characters navigate strictly through the open corridors and turns using BFS pathfinding.
  - **Dynamic Maze Refresh**: As soon as one character catches the other, the maze immediately regenerates a completely new random maze pattern with new wall layouts, glowing neon colors, character sizes, and pellet routes. This guarantees no static pixels linger on screen.
- **Iconic Odyssey² Animation Cycle & Catch Celebration**:
  1. K.C. Munchkin navigates corridors toward a pulsing **Super Pellet**, chomping normal pellets along the way.
  2. Chomping the Super Pellet triggers an expanding shockwave; K.C. gains an energized aura while the Muncher ghost turns frightened blue and flees through maze corners.
  3. K.C. aggressively chases the ghost through the corridors.
  4. The power effect fades with the ghost flashing warning colors.
  5. The ghost reverts to its normal color, turns around, and actively hunts K.C. through the maze.
  6. K.C. flees in terror with wide eyes and flying sweat drops.
  7. **When K.C. catches the ghost**:
     - The ghost's death animation plays first as it spins, shrinks, and dissolves into floating spirit bubbles and sparkles.
     - K.C. immediately turns to face the "camera", showing an adorable toothy smile with cute pink blush cheeks!
     - K.C. quickly grows arms and legs at 45° angles and excitedly pumps them up and down, bouncing with joy to cheer!
     - The maze then explodes into confetti and refreshes into a fresh random maze pattern!
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

### 1. Prerequisites: Install Rust (if not already installed)

`kcchase` requires a Rust toolchain (Rust 2024 edition or newer). If you do not have Rust installed yet, install it using the official [rustup](https://rustup.rs) installer:

```bash
# Download and install rustup (Linux & macOS)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Configure your current shell session
source "$HOME/.cargo/env"
```

Verify that the toolchain is available:
```bash
rustc --version
cargo --version
```

### 2. Install kcchase with `install.sh`

Clone the repository and run the `install.sh` script. It automatically compiles the optimized release binary and symlinks it to `~/.local/bin/kcchase`:

```bash
# Clone the repository
git clone https://github.com/AndrewAlbrightTX/kcchase.git
cd kcchase

# Run the installer
./install.sh
```

> [!NOTE]
> Ensure `~/.local/bin` is in your system's `$PATH`. If it isn't already, add the following to your `~/.bashrc`, `~/.zshrc`, or `~/.profile`:
> ```bash
> export PATH="$HOME/.local/bin:$PATH"
> ```

### Uninstallation

To remove the symlink:

```bash
./uninstall.sh
```

### Manual Build (Alternative)

If you prefer building without creating a global symlink:

```bash
cargo build --release
```

The optimized executable will be located at `target/release/kcchase`.

## Usage

If you installed via `./install.sh` (and `~/.local/bin` is in your `PATH`):

```bash
kcchase
```

Or run directly from the repository:

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
