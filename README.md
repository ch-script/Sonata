# paw-client

A fast TUI/CLI command abstractor across Unix systems. Making syntax switching trivial for distrohoppers ;)

Basically it's a little Rust CLI that helps you stop googling "how do I install a package on arch again" every time you distro-hop lol. Built for Linux and BSD, so if you're on Windows... well yk

## VERSION 0.2

### What's new?

- **Custom scripts & extensions**: You can now write your own scripts or extend the existing ones. To unlock this, you first need to allow customs on your system:

```bash
  paw --allow customs
```

  Once you run that, `paw` will spit out a path, go there and start editing the files. Check [CONFIG.md](./CONFIG.md) to see the naming/syntax you need to follow.

- **Bluetooth config support (Linux)**: You can now manage your bluetooth setup straight from `paw` instead of jumping between random tools.

- **`paw --profile`**: Shows you system info at a glance. Want everything? Run it with `--verbose`:

```bash
  paw --profile --verbose
```

  Only need one specific thing? Pass an id after it, like this:

```bash
  paw --profile --verbose --wm
```

  All of this data is stored inside a TOML file in your `.config` folder, easy to poke around if you're curious.

- **Variable support**: This is probably the highlight of 0.2. You can now set variables for your environments so you stop having to remember/type the same stuff over and over. Set one like this:

```bash
  paw --add-var <variable_name> "variable text" --target <module> <specific_module_component> --assign
```

  `--assign` is optional. If you use it, `paw` just assumes whatever variable is currently assigned for that spot. For example, instead of typing:

```bash
  paw edit write path
```

  you can just do:

```bash
  paw edit write
```

  and it'll grab the `path` variable automatically.

- **Nix support leveled up**: The declarative system is way more mature now and can handle more than one package at a time. On top of that, `paw` now supports both declarative and imperative approaches for Nix, so pick whichever fits your workflow.

- **Initial OpenRC & Gentoo compatibility**: Early support is in, more polish coming as we go (and as I learn how to even use Gentoo lol)

## Easy Install

Don't feel like reading the manual install steps below? Tap your distro and just copy-paste the commands:

- [NixOS](./installation/nixos.md) (imperative for now, flake support coming eventually)
- [Arch Linux](./installation/arch.md)
- [Solus](./installation/solus.md)
- [Ubuntu](./installation/ubuntu.md)
- [Void Linux](./installation/void.md)

Each guide installs Rust, Cargo, Git, and fish for you, then clones, builds, and installs `ch` into a clean `~/ch-software/ch-client` folder so updates stay painless. If your distro isn't listed, just follow the manual steps below, the process is basically the same everywhere anyway :D

## What you need first

Before you even think about installing it, make sure you've got these on your system:

- **Rust**
- **Cargo** (comes with Rust usually, but double check pls)
- **Git**
- **Fish-shell** (we need them for now)

Quick way to check if you already have them:

```bash
rustc --version
cargo --version
git --version
fish --version
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

Paw is momentarily bowed to Fish-Shell, so you could and most probably would experience problems on other shells.
I am working on that, since it IS a problem. It should be solved on version 0.3, so stay tuned.

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

`ch` runs off declarative TOML files, so once it's installed you can personalize the commands and submenus to your choice. Check out [CONFIG.md](./CONFIG.md) for the full syntax rundown, covers everything from basic commands to nested menus to locking down dangerous stuff behind a confirmation prompt.

## That's it

Yeah that's basically the whole setup, nothing crazy. If something breaks feel free to open an issue on the repo. :)
