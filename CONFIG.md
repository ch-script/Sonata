# Config Syntax Guide (`snt` TOML)

So `snt` is basically driven by a bunch of TOML files that get chunked together, once they're created, you can personalize them as you want. So, here is how to do it. Remember that the conf files are in this route: ~/.config/ch/config.toml

This doc breaks down the three types of nodes (modes) you can throw into your `.toml` files.

## 1. Base Categories (Root Nodes)

Every TOML file in `snt` has to start by declaring which module/category it belongs to, using square brackets `[ ]`. This tells the assembler where to inject the stuff you're defining.

```toml
# Examples of valid headers
[pkg]
[os]
[net]
```

That's it, that's the whole rule. No header, no worky.

## 2. Simple Commands

This is the most basic way to talk to `snt`. It's just a key-value pair where the value is a string (the actual command that gets run).

### Dynamic Arguments

You can use `{1}`, `{2}`, `{3}`, etc inside your command. When `snt` reads one of these, it'll pause and ask the user to type in whatever's needed before actually running the thing. Pretty handy so you don't have to hardcode a package name for every single command.

```toml
[pkg]
# Static command
update = "sudo pacman -Syu"

# Dynamic command (asks user for 1 value)
install = "sudo pacman -S {1}"

# Multi-dynamic command (asks for 2 values)
clone = "git clone {1} {2}"
```

## 3. Submenus (Nested Tables)

To keep the interface clean instead of one giant wall of commands, you can nest menus inside menus, basically forever. Just use dot `.` syntax in the category header.

Anything you declare under a submenu belongs exclusively to that node. In the interactive UI, submenus just show up as their name, nice and tidy.

```toml
# Main category
[extrapkgs]

# Submenu 1: Flatpak
[extrapkgs.flatpak]
install = "snt pkg install flatpak"
setup = "flatpak remote-add --if-not-exists flathub https://dl.flathub.org/repo/flathub.flatpakrepo"

# Submenu 2: AppImage
[extrapkgs.appimage]
install = "snt pkg install appimagelauncher"

# you can even nest deeper if you're feeling spicy
[extrapkgs.flatpak.advanced]
repair = "flatpak repair"
```

## 4. Advanced Commands (Guardrails & Confirmations)

For destructive or scary commands, `snt` supports inline tables so you can slap some safety layers on before the command actually fires. Because nobody wants to `rm -rf` the wrong thing at 2am haha right... right?

Instead of passing a plain string, you pass an object `{ }` with these fields:

- `cmd`: the actual command to run (supports `{1}` etc just like simple commands)
- `msg`: the warning message the user sees
- `confirm`: which validation method to use

### Confirm types

**A. `yesorno`**

Shows an interactive Yes/No prompt. Defaults to "No" always, so a stray Enter press doesn't nuke anything.

```toml
[pkg]
remove = { cmd = "nix-env -e {1}", confirm = "yesorno", msg = "This will remove the program and its dependencies, do you wish to continue?" }
```

**B. `match:<word>`**

User has to type the exact word after the colon for the command to actually run. Anything else, or just hitting Enter with nothing typed, aborts it. Good for the really spicy stuff.

```toml
[os]
rollback = { cmd = "sudo nixos-rebuild switch --rollback", confirm = "match:confirm", msg = "Type 'confirm' to rollback. Everything else will abort." }

# another example requiring the phrase "I-KNOW-WHAT-I-AM-DOING"
wipe_disk = { cmd = "rm -rf /", confirm = "match:I-KNOW-WHAT-I-AM-DOING", msg = "DANGER! Type the exact phrase to format disk baka" }
```

That last one is obviously just an example, please don't actually bind `rm -rf /` to anything, your future self will not thank you :c

### TCP Stream Execution (Advanced Directive)

For scenarios where you want to execute a command and stream its output or target over a specific remote network connection, `snt` supports advanced TCP stream configurations using inline tables `{ }`.

Instead of a plain string or a standard safety confirmation block, you define an object with network routing and target parameters:

- `cmd`: The actual command to run (supports `{1}` dynamic arguments just like regular commands).
- `target`: Defines the execution channel or destination type (e.g., `"remote"`).
- `port`: The target network port number for the stream.
- `ip`: The destination IP address where the stream or execution data should be directed.

#### Example

```toml
[yt]
# Executes the command and streams the video/target over a specified remote TCP connection
video = { cmd = "yt-dlp '{1}'", target = "remote", port = 9999, ip = "100.102.144.38" }
```


That's the whole syntax for now. Base category -> simple commands -> nested submenus -> optional confirm layer if you're doing something dangerous. Mix and match as needed.
