PACKAGE="$1"
PACKAGES_FILE="/etc/nixos/packages.nix"

if [ -z "$PACKAGE" ]; then
    echo "[paw] Error: Please provide a package name to remove."
    exit 1
fi

echo "[paw] Removing $PACKAGE"

if ! grep -q "^\s*$PACKAGE\s*$" "$PACKAGES_FILE"; then
    echo "[paw] Package '$PACKAGE' is not declared in $PACKAGES_FILE. So there's really nothing to do."
    exit 0
fi

echo "[paw] Creating backup of $PACKAGES_FILE"
paw --backup pkgs "$PACKAGES_FILE"

sudo sed -i "/^\s*$PACKAGE\s*$/d" "$PACKAGES_FILE"

echo "[paw] Rebuilding the system"
sudo nixos-rebuild switch
