#!/usr/bin/env sh
set -e

echo "Installing Paw"

INSTALL_DIR="$HOME/.local/bin"
BIN_NAME="paw"

# Ensure local bin directory exists
mkdir -p "$INSTALL_DIR"

# Build the binary
if [ -f "Cargo.toml" ] && grep -q "workspace" "Cargo.toml"; then
    echo "[paw] Cargo workspace detected. Building from source"

    if ! command -v cargo >/dev/null 2>&1; then
        echo "[paw] Error: 'cargo' is not installed. Please install Rust first."
        exit 1
    fi

    cargo build --release -p interface
    echo "[paw] Copying binary to $INSTALL_DIR..."
    cp target/release/paw "$INSTALL_DIR/$BIN_NAME"
else
    echo "[paw] Error: Please run this script from inside the cloned Paw repository."
    exit 1
fi

chmod +x "$INSTALL_DIR/$BIN_NAME"

# Path Configuration
if ! echo "$PATH" | grep -q "$INSTALL_DIR"; then
    echo "[paw] Directory $INSTALL_DIR is not in your PATH. Adding it now"

    # available shell configurations
    for conf in "$HOME/.bashrc" "$HOME/.zshrc" "$HOME/.profile"; do
        if [ -f "$conf" ]; then
            if ! grep -q "$INSTALL_DIR" "$conf"; then
                echo "" >> "$conf"
                echo "# Added by Paw Installer" >> "$conf"
                echo "export PATH=\"$INSTALL_DIR:\$PATH\"" >> "$conf"
                echo "[paw] Successfully updated $conf"
            fi
        fi
    done

    echo "[paw] Note: You might need to restart your terminal or run 'source ~/.bashrc' (or your shell config)."
else
    echo "[paw] $INSTALL_DIR is already in your PATH."
fi

echo "Paw installed successfully! Try running 'paw --create' to initialize."
