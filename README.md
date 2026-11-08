# kcchase

A high-resolution retro terminal screensaver inspired by the classic Magnavox Odyssey² / Philips Videopac game **K.C. Munchkin!** (1981).

## Features

- **Iconic Odyssey² Animation Cycle**:
  1. K.C. Munchkin approaches and chomps a pulsing **Super Pellet**, triggering a shockwave burst.
  2. K.C. gains an energetic aura while the Muncher ghost turns frightened blue and flees in panic.
  3. K.C. aggressively chases the ghost around the arena while gobbling pellets.
  4. The super-charge effect begins to wear off; the ghost frantically flashes between blue and bright warning colors.
  5. The effect fades completely: the ghost reverts to its menacing color, turns around, and actively hunts K.C.!
  6. K.C. flees in terror with wide eyes and sweat flying, before escaping off-screen.
  7. On each pass, the characters can vary in size, randomize the ghost (Bit, Byte, Middie, or Orange), reposition the super pellet, and generate a new arena layout.
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
