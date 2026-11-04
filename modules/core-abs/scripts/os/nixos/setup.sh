CONFIG_FILE="/etc/nixos/configuration.nix"
PACKAGES_FILE="/etc/nixos/packages.nix"

echo "[snt] Setting up declarative environment in $CONFIG_FILE"

if grep -q "\./packages.nix" "$CONFIG_FILE"; then
    echo "[snt] ./packages.nix is already in your imports. All good!!"
else
    echo "[snt] Creating backup of $CONFIG_FILE"
    snt --backup pkgs "$CONFIG_FILE"
    sudo sed -i '/imports =/,/]/ s/^\(\s*\)]/\1  .\/packages.nix\n\1]/' "$CONFIG_FILE"
    echo "[snt] ./packages.nix injected successfully."
fi

if [ ! -f "$PACKAGES_FILE" ]; then
    sudo tee "$PACKAGES_FILE" > /dev/null <<EOF
{ pkgs, ... }:

{
  environment.systemPackages = with pkgs; [
    # ch_packages_start
    # ch_packages_end
  ];
}
EOF
    echo "[snt] $PACKAGES_FILE created successfully."
fi
