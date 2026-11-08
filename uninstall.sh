#!/usr/bin/env bash
set -euo pipefail

TARGET_LINK="${HOME}/.local/bin/kcchase"

if [ -L "${TARGET_LINK}" ] || [ -f "${TARGET_LINK}" ]; then
    echo "==> Removing symlink: ${TARGET_LINK}"
    rm -f "${TARGET_LINK}"
    echo "Successfully uninstalled kcchase from ${TARGET_LINK}"
else
    echo "kcchase is not installed at ${TARGET_LINK}"
fi
