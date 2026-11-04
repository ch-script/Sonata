# Installing ch-client on Void Linux

Enter the void, viva españa, copy these commands:

## 0. Install the dependencies

```bash
sudo xbps-install -Sy rustup git fish-shell pciutils micro
rustup default stable
fish_add_path ~/.cargo/bin
```

Heads up: on Void the fish package is called `fish-shell`, not `fish` (the command itself is still just `fish` once it's in). `pciutils` gets you `lspci`, `micro` is a comfy little terminal editor for your dots.

## Install ch-client

Just copy and paste this on your terminal

```bash
rm -rf ~/ch-software/ch-client
mkdir -p ~/ch-software
cd ~/ch-software
git clone https://github.com/ch-script/ch-client.git
cd ch-client
cargo build --release
mkdir -p ~/.local/bin
cp target/release/ch ~/.local/bin/
chmod +x ~/.local/bin/ch
set -U fish_user_paths ~/.local/bin $fish_user_paths
source ~/.config/fish/config.fish
ch --create
```

If it spits something out instead of "command not found", you're done. Go set up your dots now, see [CONFIG.md](../CONFIG.md).