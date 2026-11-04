#!/usr/bin/env bash
set -euo pipefail

PACKAGES_FILE="/etc/nixos/packages.nix"

package_exists() {
    local pkg="$1"
    nix-instantiate --eval -E "with import <nixpkgs> {}; ${pkg}.name" >/dev/null 2>&1
}

if [ "$#" -eq 0 ]; then
    echo "[snt] Error: Please provide at least one package name."
    exit 1
fi

NEW_PACKAGES=()

for PACKAGE in "$@"; do
    if grep -qE "^\s*${PACKAGE}\s*\$" "$PACKAGES_FILE"; then
        echo "[snt] Package '$PACKAGE' is already declared. Skipping."
        continue
    fi

    echo "[snt] Checking if '$PACKAGE' exists in nixpkgs..."
    if ! package_exists "$PACKAGE"; then
        echo "[snt] Warning: '$PACKAGE' not found in nixpkgs. Skipping."
        continue
    fi

    NEW_PACKAGES+=("$PACKAGE")
done

if [ "${#NEW_PACKAGES[@]}" -eq 0 ]; then
    echo "[snt] Nothing to do."
    exit 0
fi

echo "[snt] Creating backup of $PACKAGES_FILE"
snt --backup pkgs "$PACKAGES_FILE"

for PACKAGE in "${NEW_PACKAGES[@]}"; do
    echo "[snt] Installing $PACKAGE"
    sudo sed -i "/# ch_packages_end/i\\    $PACKAGE" "$PACKAGES_FILE"
done

echo "[snt] Rebuilding the system"
sudo nixos-rebuild switch
