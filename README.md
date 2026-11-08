# kcchase

A high-resolution retro terminal screensaver inspired by the classic 
Magnavox Odyssey² / Philips Videopac game **K.C. Munchkin!** (1981).
Pretty much a screensaver game that plays itself. Includes cut-
scene after KC dies twice to clear the screen for a bit, then
resumes game mode. Score tallies, but is not written to disk, so 
hi-score starts at 0 on each launch.

## Features

- **Procedural Arcade Mazes**:
- **Full Terminal Screen Fitting & Dynamic Resizing**:
- **Color Capability Detection**:
- **Screensaver Style Exit**:

## Installation & Building

### 1. Prerequisites: Install Rust (if not already installed)

`kcchase` requires a Rust toolchain (Rust 2024 edition or newer). If you do not
have Rust installed yet, install it using the official 
[rustup](https://rustup.rs) installer:

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

Clone the repository and run the `install.sh` script. It automatically compiles
 the optimized release binary and symlinks it to `~/.local/bin/kcchase`:

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

If you prefer building without creating a local installed symlink:

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
    -v, --version            Print version information and exit
    -i, --info               Display detected terminal color capabilities and exit
    -c, --color <MODE>       Select color mode: truecolor, 256, 16, 8 (default: auto)
    --fps <NUMBER>           Set target frame rate (default: 60, range: 15-120)
    --speed <FLOAT>          Set animation speed multiplier (default: 1.0)
    -k, --keyboard           Exit on keyboard input only (ignores mouse movement)
    -m, --mouse              Exit on mouse movement only (ignores keyboard, except Ctrl+C/Esc)
    --intermission, --cutscene Start with the intermission animation sequence
    --game                     Start with the game sequence (default)
    --sequence <MODE>          Select starting sequence: game (default), intermission

EXIT:
    Press any key or move the mouse to exit the screensaver.
    Use --keyboard or --mouse to restrict the exit trigger.
```

### Screensaver Anti-Burn-In & Intermission Sequence

To protect CRT and modern displays against screen burn, `kcchase` features:
- **Zero Static Elements**: Maze layouts, corridor wall colors, and pellet positions regenerate after every pass. Score and high scores are tracked strictly in memory.
- **Edge Rest Intermission Sequence**: After 2 levels (unless K.C. is on a winning streak), the maze walls, pellets, and HUD completely clear to give the edges of the display a full rest.
- **Horizontal Chase Cutscenes**: Scaled-up sprites cross horizontally in alternating sweeps:
  - K.C. supercharged chasing the 3 vulnerable ghosts across the screen.
  - K.C. running terrified back in the opposite direction on a different horizontal line with the three hunting ghosts (Bit, Middie, Byte) pursuing right behind him.
  - Repeats for at least 6 full cycles across varying horizontal lines before returning to a fresh procedural maze game pass.
- **Starting Sequence Control**: Use `--intermission` or `--cutscene` to boot directly into the horizontal intermission animation, or `--game` to start directly in the maze. Sequences continue rotating automatically.

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
