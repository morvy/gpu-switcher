# gpu-switcher

KDE Plasma 6 widget + Rust system daemon for switching AMD GPU performance profiles on laptops with RDNA 3.5 / Strix Halo APUs (Radeon 8050S, 8060S, and similar).

Controls `power_dpm_force_performance_level` and couples it with `power-profiles-daemon` in a single 5-tier slider. Supports automatic profile switching based on AC/battery state or battery percentage threshold.

---

## Features

- 5-tier profile slider (Save → Adaptive → Standard → Perf → Max)
- Couples AMD GPU sysfs level with power-profiles-daemon profile in lockstep
- Auto-switch modes: manual, AC/battery, or battery percentage threshold
- Tray icon changes color per profile (dark blue → teal → amber → orange → red)
- Lightweight: ~2 MB Rust daemon, idle CPU ≈ 0%, pure QML widget (no compiled C++)
- Config persisted at `/etc/gpu-switcher.toml`

## Profile Mapping

| Stop | Label    | AMDGPU level       | PPD profile   | Use case             |
|------|----------|--------------------|---------------|----------------------|
| 1    | Save     | `low`              | power-saver   | Max battery life     |
| 2    | Adaptive | `auto`             | balanced      | Kernel-managed (default) |
| 3    | Standard | `profile_standard` | balanced      | Stable mid clock     |
| 4    | Perf     | `high`             | performance   | Sustained workloads  |
| 5    | Max      | `profile_peak`     | performance   | Peak boost           |

## Requirements

- KDE Plasma 6.x
- `plasma5support` package (for `P5Support.DataSource` in QML)
- `power-profiles-daemon` (running)
- `busctl` (part of `systemd`)
- AMD GPU with `power_dpm_force_performance_level` sysfs node (RDNA 3.x / Strix Halo confirmed)
- Rust toolchain (for building from source)

Arch / CachyOS:
```sh
sudo pacman -S plasma5support power-profiles-daemon
```

## Installation

### From source

```sh
git clone https://github.com/morvy/gpu-switcher
cd gpu-switcher

# Build daemon + install all system files (requires sudo)
sudo packaging/install.sh

# Enable and start the daemon
sudo systemctl enable --now gpu-switcher

# Install the Plasma widget
kpackagetool6 --type=Plasma/Applet -i plasmoid/package
```

Then right-click your panel → **Add Widgets** → search **GPU Profile Switcher**.

### Verify

```sh
# Daemon health
systemctl status gpu-switcher

# Manual test via CLI
gpu-switcher-daemon get
gpu-switcher-daemon set 5   # switch to Max
cat /sys/class/drm/card*/device/power_dpm_force_performance_level
powerprofilesctl get
```

## CLI Usage

The daemon binary doubles as a CLI client:

```sh
gpu-switcher-daemon daemon   # run as system service (used by systemd)
gpu-switcher-daemon get      # print current stop (1-5)
gpu-switcher-daemon set 3    # set profile to Standard
```

## Configuration

Edit `/etc/gpu-switcher.toml` (or let the widget write it):

```toml
[profile]
current_stop = 2      # 1-5, last active stop
ac_default_stop = 3   # stop to apply when on AC power
battery_stop = 1      # stop to apply when on battery / below threshold

[auto]
mode = "manual"          # "manual" | "ac_battery" | "battery_pct"
battery_threshold = 30   # percent, used by battery_pct mode
```

### Auto-switch modes

| Mode          | Behaviour                                                   |
|---------------|-------------------------------------------------------------|
| `manual`      | No automatic switching; slider is the only control          |
| `ac_battery`  | Switches to `battery_stop` on unplug; `ac_default_stop` on AC |
| `battery_pct` | Drops to `battery_stop` when battery < `battery_threshold`% |

Configure via the widget's settings dialog (right-click → Configure GPU Profile Switcher).

## Architecture

```
Plasma panel
└── gpu-switcher plasmoid (pure QML)
    └─── polls busctl every 3 s for CurrentStop
    └─── calls busctl SetStop on slider move
         │ system DBus
         ▼
    gpu-switcher-daemon (Rust, runs as root)
    ├── writes /sys/.../power_dpm_force_performance_level
    ├── calls net.hadess.PowerProfiles.SetActiveProfile
    └── watches org.freedesktop.UPower for auto-switch events
```

## Building from Source

```sh
# Requires Rust stable (rustup recommended)
cargo build --release -p gpu-switcher-daemon

# Binary is at target/release/gpu-switcher-daemon
```

## Uninstall

```sh
# Remove plasmoid
kpackagetool6 --type=Plasma/Applet -r net.gpuswitcher.applet

# Stop and disable daemon
sudo systemctl disable --now gpu-switcher

# Remove system files
sudo rm /usr/local/bin/gpu-switcher-daemon
sudo rm /usr/lib/systemd/system/gpu-switcher.service
sudo rm /usr/share/dbus-1/system.d/net.gpuswitcher.Manager.conf
sudo rm /usr/share/dbus-1/system-services/net.gpuswitcher.Manager.service
sudo rm /usr/share/polkit-1/actions/net.gpuswitcher.policy
sudo rm -f /etc/gpu-switcher.toml
sudo systemctl daemon-reload
```

## Compatibility

Tested on:
- **AMD Strix Halo** (Radeon 8050S / 8060S, RDNA 3.5) — CachyOS, KDE Plasma 6.6.4

Should work on any AMD dGPU or APU that exposes `power_dpm_force_performance_level`. Not tested on NVIDIA or Intel GPUs.

Note: `pp_power_profile_mode` (fine-grained DPM heuristics) is not exposed on Strix Halo APUs and is therefore not used.

## Known Limitations / TODO

- Polkit check not yet enforced inside the daemon (DBus policy restricts to console-session users; v2 will call `pkcheck` per method)
- Single GPU only (first AMDGPU node discovered)
- No NVIDIA / Intel path
- Translations: English only

## License

GPL-2.0-or-later — see [LICENSE](LICENSE).
