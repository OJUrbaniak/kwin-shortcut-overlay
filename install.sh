#!/bin/bash
set -e

echo "=========================================="
echo "   KWin Shortcut Overlay Installer & Setup"
echo "=========================================="

# 1. Build the binary
echo "--> Compiling Rust binary in release mode..."
cargo build --release

# 2. Install the binary locally
echo "--> Installing binary to ~/.local/bin..."
mkdir -p "$HOME/.local/bin"
cp target/release/kwin-shortcut-overlay "$HOME/.local/bin/kwin-shortcut-overlay"
chmod +x "$HOME/.local/bin/kwin-shortcut-overlay"

# 3. Create a standard .desktop entry for Plasma 6 (StartupNotify disabled)
echo "--> Creating desktop entry for the shortcut..."
mkdir -p "$HOME/.local/share/applications"
cat <<EOF > "$HOME/.local/share/applications/kwin-shortcut-overlay.desktop"
[Desktop Entry]
Name=KWin Shortcut Overlay
Exec=$HOME/.local/bin/kwin-shortcut-overlay
Icon=preferences-desktop-keyboard-shortcuts
Type=Application
Terminal=false
Categories=Utility;
X-KDE-Shortcuts=Meta+/
StartupNotify=false
EOF

# 4. Notify the desktop environment to parse the new application
echo "--> Updating desktop database..."
update-desktop-database "$HOME/.local/share/applications" || true

# 5. Tell kglobalaccel to reload the shortcuts immediately
echo "--> Registering shortcut in Plasma 6..."
kwriteconfig6 --file kglobalshortcutsrc --group "kwin-shortcut-overlay.desktop" --key "_k_friendly_name" "KWin Shortcut Overlay"
kwriteconfig6 --file kglobalshortcutsrc --group "kwin-shortcut-overlay.desktop" --key "_launch" "Meta+/,none,KWin Shortcut Overlay"

# Refresh the daemon natively
systemctl --user restart plasma-kglobalaccel.service || true

echo "=========================================="
echo " Success!"
echo " Press Meta + / to launch."
echo "=========================================="
