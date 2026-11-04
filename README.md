# paw-client

A fast TUI/CLI command abstractor across Unix systems. Making syntax switching trivial for distrohoppers ;)

Basically it's a little Rust CLI that helps you stop googling "how do I install a package on arch again" every time you distro-hop lol. Built for Linux and BSD, so if you're on Windows... well yk

## Version History

- [0.2](./docs/version/0.2.md)
- [0.3](./docs/version/0.3.md)

## Ideas and vision

[Read](./docs/visions/Vision.md)

## Easy Install

Run this command inside the installation folder
```bash
chmod +x install.sh
./install.sh
```

## What you need first (hard dependencies)

Before you even think about installing it, make sure you've got these on your system:

- **Rust**
- **Cargo** (comes with Rust usually, but double check pls)
- **Git**

Quick way to check if you already have them:

```bash
rustc --version
cargo --version
git --version
```

If any of those spit out an error instead of a version number, go grab [rustup](https://rustup.rs/) first, it installs Rust + Cargo together in one go.
Or use ur distro's package manager, most have it.

## Installing it

### 1. Clone the repo

```bash
git clone https://github.com/ch-script/ch-client.git
```

### 2. Get into the folder

```bash
cd ch-client
```

### 3. Build it

```bash
cargo build --release
```

This'll take a sec depending on your machine, cargo's compiling everything in release mode so it's optimized and fast.

### 4. Move the binary somewhere useful

Once the build's done, the binary will be chilling at `target/release/ch`. Send it to your local bin so it's actually usable from anywhere:

```bash
mkdir -p ~/.local/bin
cp target/release/ch ~/.local/bin/
```

### 5. Add it to your shell path

If you're rocking fish (the correct choice), you gotta tell it where to find the binary. Run this:

```fish
set -U fish_user_paths ~/.local/bin $fish_user_paths
```

That `-U` makes it universal so it persists across sessions, you won't have to redo this every time you open a new terminal.

If you're using a different shell (bash/zsh), just add this to your `.bashrc` or `.zshrc` instead:

```bash
export PATH="$HOME/.local/bin:$PATH"
```

### 6. Reload your shell (or just open a new terminal)

```fish
source ~/.config/fish/config.fish
```

or just close and reopen your terminal, works too lol

## Verify it works

```bash
ch
```

If it prints something back instead of "command not found", you're good

## Do you want to personalize it?

`paw` runs off declarative TOML files, so once it's installed you can personalize the commands and submenus to your choice. Check out [CONFIG.md](./CONFIG.md) for the full syntax rundown, covers everything from basic commands to nested menus to locking down dangerous stuff behind a confirmation prompt.

## That's it

Yeah that's basically the whole setup, nothing crazy. If something breaks feel free to open an issue on the repo. :)
