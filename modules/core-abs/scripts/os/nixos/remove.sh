PACKAGE="$1"
PACKAGES_FILE="/etc/nixos/packages.nix"

if [ -z "$PACKAGE" ]; then
    echo "[snt] Error: Please provide a package name to remove."
    exit 1
fi

echo "[snt] Removing $PACKAGE"

if ! grep -q "^\s*$PACKAGE\s*$" "$PACKAGES_FILE"; then
    echo "[snt] Package '$PACKAGE' is not declared in $PACKAGES_FILE. So there's really nothing to do."
    exit 0
fi

echo "[snt] Creating backup of $PACKAGES_FILE"
snt --backup pkgs "$PACKAGES_FILE"

sudo sed -i "/^\s*$PACKAGE\s*$/d" "$PACKAGES_FILE"

echo "[snt] Rebuilding the system"
sudo nixos-rebuild switch
