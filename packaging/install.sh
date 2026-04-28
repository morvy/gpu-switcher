#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"

echo "Building gpu-switcher-daemon..."
# Build as the invoking user (sudo drops PATH; cargo must run as user)
if [ -n "${SUDO_USER:-}" ]; then
    su -l "$SUDO_USER" -c "cd '$ROOT_DIR' && cargo build --release -p gpu-switcher-daemon"
else
    cargo build --release --manifest-path "$ROOT_DIR/Cargo.toml" -p gpu-switcher-daemon
fi

echo "Installing daemon binary..."
install -Dm755 "$ROOT_DIR/target/release/gpu-switcher-daemon" /usr/local/bin/gpu-switcher-daemon

echo "Installing systemd service..."
install -Dm644 "$SCRIPT_DIR/gpu-switcher.service" /usr/lib/systemd/system/gpu-switcher.service

echo "Installing DBus policy..."
install -Dm644 "$SCRIPT_DIR/net.gpuswitcher.Manager.conf" /usr/share/dbus-1/system.d/net.gpuswitcher.Manager.conf

echo "Installing DBus activation..."
install -Dm644 "$SCRIPT_DIR/net.gpuswitcher.Manager.service" /usr/share/dbus-1/system-services/net.gpuswitcher.Manager.service

echo "Installing polkit action..."
install -Dm644 "$SCRIPT_DIR/net.gpuswitcher.policy" /usr/share/polkit-1/actions/net.gpuswitcher.policy

echo "Reloading systemd and DBus..."
systemctl daemon-reload
dbus-send --system --type=method_call --dest=org.freedesktop.DBus / org.freedesktop.DBus.ReloadConfig 2>/dev/null || true

echo "Done. Enable with: systemctl enable --now gpu-switcher"
echo "Install plasmoid with: kpackagetool6 --type=Plasma/Applet -i plasmoid/package"
