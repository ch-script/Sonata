# Installing ch-client on Arch Linux

You use arch btw, you probably already have half of this installed, but let's not assume lol. Automatic installation, copy these commands:

## 0. Install the dependencies

```bash
sudo pacman -Syu --needed rustup git fish pciutils micro
rustup default stable
fish_add_path ~/.cargo/bin
```

`--needed` just skips reinstalling stuff you already have, no reason to waste time recompiling your whole system for no reason. `pciutils` gives you `lspci`, and `micro` is just a comfy little terminal text editor for when you're writing your dots.

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

:)

If it spits something out instead of "command not found", you're done. Go set up your dots now, see [CONFIG.md](../CONFIG.md).