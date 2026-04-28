use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

const CONFIG_PATH: &str = "/etc/gpu-switcher.toml";

#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AutoMode {
    Manual,
    AcBattery,
    BatteryPct,
}

impl Default for AutoMode {
    fn default() -> Self {
        AutoMode::Manual
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProfileConfig {
    pub current_stop: u8,
    pub ac_default_stop: u8,
    pub battery_stop: u8,
}

impl Default for ProfileConfig {
    fn default() -> Self {
        ProfileConfig {
            current_stop: 2,
            ac_default_stop: 3,
            battery_stop: 1,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AutoConfig {
    pub mode: AutoMode,
    pub battery_threshold: u8,
}

impl Default for AutoConfig {
    fn default() -> Self {
        AutoConfig {
            mode: AutoMode::Manual,
            battery_threshold: 30,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    #[serde(default)]
    pub profile: ProfileConfig,
    #[serde(default)]
    pub auto: AutoConfig,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            profile: ProfileConfig::default(),
            auto: AutoConfig::default(),
        }
    }
}

impl Config {
    /// Validate field ranges after loading from disk.
    pub fn validate(&self) -> Result<()> {
        for (name, val) in [
            ("current_stop", self.profile.current_stop),
            ("ac_default_stop", self.profile.ac_default_stop),
            ("battery_stop", self.profile.battery_stop),
        ] {
            if !(1..=5).contains(&val) {
                bail!("config: {} must be 1–5, got {}", name, val);
            }
        }
        if !(1..=99).contains(&self.auto.battery_threshold) {
            bail!(
                "config: battery_threshold must be 1–99, got {}",
                self.auto.battery_threshold
            );
        }
        Ok(())
    }

    /// Load from /etc/gpu-switcher.toml; fall back to defaults if the file is absent.
    pub fn load() -> Result<Config> {
        let path = Path::new(CONFIG_PATH);
        if !path.exists() {
            return Ok(Config::default());
        }
        let content =
            fs::read_to_string(path).context("failed to read /etc/gpu-switcher.toml")?;
        let config: Config =
            toml::from_str(&content).context("failed to parse /etc/gpu-switcher.toml")?;
        config.validate()?;
        Ok(config)
    }

    /// Save to /etc/gpu-switcher.toml.
    pub fn save(&self) -> Result<()> {
        let content =
            toml::to_string_pretty(self).context("failed to serialize config")?;
        fs::write(CONFIG_PATH, content).context("failed to write /etc/gpu-switcher.toml")
    }
}
