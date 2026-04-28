# Contributing

## Reporting Issues

Please include:
- `uname -r` (kernel version)
- `plasmashell --version`
- `lspci -nn | grep VGA`
- `cat /sys/class/drm/card*/device/power_dpm_force_performance_level`
- `systemctl status gpu-switcher`
- `journalctl -u gpu-switcher -n 50`

## Development

```sh
git clone https://github.com/morvy/gpu-switcher
cd gpu-switcher

# Build daemon in debug mode
cargo build -p gpu-switcher-daemon

# Run tests
cargo test -p gpu-switcher-daemon

# Iterate on the plasmoid without reinstalling
plasmoidviewer -a plasmoid/package
```

## Code Style

- Rust: `cargo fmt` + `cargo clippy --deny warnings`
- QML: 4-space indent, no trailing whitespace

## Architecture Notes

- Daemon modules are independent: `sysfs`, `config`, `ppd`, `upower`, `automode` have no circular deps
- `dbus.rs` wires them together; `main.rs` owns the tokio runtime
- QML widget talks to daemon only via `busctl` (no compiled plugin)
- All sysfs writes go through `AmdgpuNode::set_stop` which validates input; never write to arbitrary paths

## TODOs Welcome

- Polkit enforcement (`pkcheck` call in `SetStop`/`SetAutoMode`)
- Per-device subscription for battery percentage drain events (currently watches UPower root + battery device)
- Multi-GPU support
- Package for AUR
