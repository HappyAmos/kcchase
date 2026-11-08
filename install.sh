#!/usr/bin/env bash
set -euo pipefail

# Determine repository root directory
REPO_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BIN_PATH="${REPO_DIR}/target/release/kcchase"
TARGET_DIR="${HOME}/.local/bin"
TARGET_LINK="${TARGET_DIR}/kcchase"

echo "==> Building kcchase (release mode)..."
cargo build --release --manifest-path "${REPO_DIR}/Cargo.toml"

if [ ! -f "${BIN_PATH}" ]; then
    echo "Error: Binary not found at ${BIN_PATH}" >&2
    exit 1
fi

echo "==> Ensuring ${TARGET_DIR} exists..."
mkdir -p "${TARGET_DIR}"

echo "==> Creating symlink: ${TARGET_LINK} -> ${BIN_PATH}"
ln -sf "${BIN_PATH}" "${TARGET_LINK}"

echo ""
echo "Successfully installed kcchase!"
echo "Symlink created: ${TARGET_LINK} -> ${BIN_PATH}"

# Check if ~/.local/bin is in PATH
if [[ ":$PATH:" != *":${TARGET_DIR}:"* ]]; then
    echo ""
    echo "Note: ${TARGET_DIR} is not currently in your PATH."
    echo "You can add it by appending this line to your ~/.bashrc or ~/.profile:"
    echo '  export PATH="$HOME/.local/bin:$PATH"'
fi

echo ""
echo "Run 'kcchase' in any terminal to start the screensaver!"
