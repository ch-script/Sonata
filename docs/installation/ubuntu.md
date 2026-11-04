# Installing ch-client on Ubuntu

Ubuntu 24.04+ finally ships rustup in the official repos, so no more piping random scripts from the internet into `sh` lol. Automatic installation, copy these commands:

## 0. Install the dependencies

```bash
sudo apt update
sudo apt install -y rustup git fish build-essential pciutils micro
rustup default stable
fish_add_path ~/.cargo/bin
```

`build-essential` gets you gcc and friends, cargo needs a linker to actually compile anything. `pciutils` gives you `lspci`, and `micro` is just a comfy terminal editor for when you're writing your dots.

> Running an older Ubuntu (22.04 or earlier) and `rustup` isn't found via apt? Just grab it straight from upstream instead: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`, then run the `rustup default stable` and `fish_add_path` lines above.

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