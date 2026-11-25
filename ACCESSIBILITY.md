# Accessibility Statement

We aim to make **kcchase** usable across a wide range of terminals, display setups, and input preferences.

## Supported Accessibility Features

- **Flexible Exit Controls:** By default, any keypress or mouse movement exits the screensaver immediately. Users who prefer or require single-modality input can restrict exit triggers with `-k` / `--keyboard` (keyboard only) or `-m` / `--mouse` (mouse movement only, with `Ctrl+C` and `Esc` always preserved as reliable keyboard fallbacks).
- **Color & Contrast Fallbacks:** `kcchase` automatically detects terminal color depth and also allows explicit selection via `-c` / `--color <MODE>` across `truecolor` (24-bit), `256` (8-bit), `16` (4-bit), and `8` (3-bit) palettes so it degrades cleanly in lower-color or high-contrast terminal themes.
- **Motion & Frame-Rate Controls:** Users sensitive to fast animation can lower the frame rate (`--fps <15-120>`) or slow animation speed (`--speed <FLOAT>`, e.g., `--speed 0.5`).
- **Terminal State Safety:** `kcchase` uses an RAII terminal guard to cleanly restore the cursor, primary screen buffer, autowrap mode, and input state on exit.
- **Plain-Text CLI Output:** `--help`, `--version`, and `--info` output plain, screen-reader-friendly text without ANSI graphics or raw terminal takeover.

## Known Limitations

- `kcchase` is primarily a visual terminal screensaver rendered with Unicode quadrant block characters and animated color sprites; its live animation is not designed for screen readers.
- Energized chase states and wall color transitions include visual animation and color changes. Reducing `--fps` and `--speed` can help moderate visual intensity.

## Reporting Accessibility Barriers

If you encounter an accessibility issue or have suggestions for improving usability (for example, additional colorblind-friendly palette options or reduced-motion presets), please [open an issue](https://github.com/HappyAmos/kcchase/issues/new/choose) and describe your terminal emulator, OS, and the barrier encountered.
