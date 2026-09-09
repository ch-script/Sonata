#!/usr/bin/env bash
set -euo pipefail

PACKAGES_FILE="/etc/nixos/packages.nix"

package_exists() {
    local pkg="$1"
    nix-instantiate --eval -E "with import <nixpkgs> {}; ${pkg}.name" >/dev/null 2>&1
}

if [ "$#" -eq 0 ]; then
    echo "[paw] Error: Please provide at least one package name."
    exit 1
fi

NEW_PACKAGES=()

for PACKAGE in "$@"; do
    if grep -qE "^\s*${PACKAGE}\s*\$" "$PACKAGES_FILE"; then
        echo "[paw] Package '$PACKAGE' is already declared. Skipping."
        continue
    fi

    echo "[paw] Checking if '$PACKAGE' exists in nixpkgs..."
    if ! package_exists "$PACKAGE"; then
        echo "[paw] Warning: '$PACKAGE' not found in nixpkgs. Skipping."
        continue
    fi

    NEW_PACKAGES+=("$PACKAGE")
done

if [ "${#NEW_PACKAGES[@]}" -eq 0 ]; then
    echo "[paw] Nothing to do."
    exit 0
fi

echo "[paw] Creating backup of $PACKAGES_FILE"
paw --backup pkgs "$PACKAGES_FILE"

for PACKAGE in "${NEW_PACKAGES[@]}"; do
    echo "[paw] Installing $PACKAGE"
    sudo sed -i "/# ch_packages_end/i\\    $PACKAGE" "$PACKAGES_FILE"
done

echo "[paw] Rebuilding the system"
sudo nixos-rebuild switch
