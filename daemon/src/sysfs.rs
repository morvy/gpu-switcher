use anyhow::{Context, Result, anyhow};
use std::fs;
use std::path::PathBuf;

const VALID_AMDGPU_LEVELS: &[&str] = &["low", "auto", "profile_standard", "high", "profile_peak"];

/// Maps a profile stop (1–5) to its AMDGPU power_dpm_force_performance_level string.
pub fn stop_to_amdgpu(stop: u8) -> Result<&'static str> {
    match stop {
        1 => Ok("low"),
        2 => Ok("auto"),
        3 => Ok("profile_standard"),
        4 => Ok("high"),
        5 => Ok("profile_peak"),
        _ => Err(anyhow!("invalid stop {}: must be 1–5", stop)),
    }
}

/// Maps a profile stop (1–5) to the power-profiles-daemon profile string.
pub fn stop_to_ppd(stop: u8) -> Result<&'static str> {
    match stop {
        1 => Ok("power-saver"),
        2 | 3 => Ok("balanced"),
        4 | 5 => Ok("performance"),
        _ => Err(anyhow!("invalid stop {}: must be 1–5", stop)),
    }
}

pub struct AmdgpuNode {
    perf_level_path: PathBuf,
}

impl AmdgpuNode {
    /// Discover the first AMDGPU card's power_dpm_force_performance_level sysfs path.
    /// Walks /sys/class/drm/card*/device, checks uevent for DRIVER=amdgpu.
    pub fn discover() -> Result<Self> {
        let drm_path = PathBuf::from("/sys/class/drm");
        let mut entries: Vec<_> = fs::read_dir(&drm_path)
            .context("failed to read /sys/class/drm")?
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.file_name()
                    .to_string_lossy()
                    .starts_with("card")
            })
            .collect();

        entries.sort_by_key(|e| e.file_name());

        for entry in entries {
            let card_path = entry.path();
            let uevent_path = card_path.join("device/uevent");

            let uevent = match fs::read_to_string(&uevent_path) {
                Ok(s) => s,
                Err(_) => continue,
            };

            if uevent.lines().any(|l| l == "DRIVER=amdgpu") {
                let perf_level_path =
                    card_path.join("device/power_dpm_force_performance_level");
                return Ok(AmdgpuNode { perf_level_path });
            }
        }

        Err(anyhow!("no AMDGPU card found in /sys/class/drm"))
    }

    /// Write the given stop's AMDGPU level string to sysfs.
    pub fn set_stop(&self, stop: u8) -> Result<()> {
        if !(1..=5).contains(&stop) {
            return Err(anyhow!("invalid stop {}: must be 1–5", stop));
        }
        let level = stop_to_amdgpu(stop)?;
        debug_assert!(
            VALID_AMDGPU_LEVELS.contains(&level),
            "stop_to_amdgpu returned invalid level: {}",
            level
        );
        fs::write(&self.perf_level_path, level).with_context(|| {
            format!(
                "failed to write '{}' to {}",
                level,
                self.perf_level_path.display()
            )
        })
    }

    /// Read the current raw level string from sysfs.
    pub fn read_raw(&self) -> Result<String> {
        let raw = fs::read_to_string(&self.perf_level_path).with_context(|| {
            format!(
                "failed to read {}",
                self.perf_level_path.display()
            )
        })?;
        Ok(raw.trim().to_string())
    }
}
