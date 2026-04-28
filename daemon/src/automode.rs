use crate::config::{AutoMode, Config};
use crate::upower::UPowerState;

/// Given current auto-mode config and UPower state, compute the desired profile stop.
/// Returns None if mode is Manual (caller should not change current stop).
pub fn compute_desired_stop(config: &Config, state: &UPowerState) -> Option<u8> {
    match config.auto.mode {
        AutoMode::Manual => None,
        AutoMode::AcBattery => {
            if state.on_battery {
                Some(config.profile.battery_stop)
            } else {
                Some(config.profile.ac_default_stop)
            }
        }
        AutoMode::BatteryPct => {
            let pct = state.battery_percentage.unwrap_or(100);
            if pct < config.auto.battery_threshold {
                Some(config.profile.battery_stop)
            } else {
                Some(config.profile.ac_default_stop)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{AutoConfig, AutoMode, Config, ProfileConfig};

    fn make_config(mode: AutoMode, threshold: u8) -> Config {
        Config {
            profile: ProfileConfig {
                current_stop: 3,
                ac_default_stop: 4,
                battery_stop: 1,
            },
            auto: AutoConfig {
                mode,
                battery_threshold: threshold,
            },
        }
    }

    #[test]
    fn manual_returns_none() {
        let cfg = make_config(AutoMode::Manual, 30);
        let state = UPowerState { on_battery: true, battery_percentage: Some(10) };
        assert_eq!(compute_desired_stop(&cfg, &state), None);
    }

    #[test]
    fn ac_battery_on_battery() {
        let cfg = make_config(AutoMode::AcBattery, 30);
        let state = UPowerState { on_battery: true, battery_percentage: Some(80) };
        assert_eq!(compute_desired_stop(&cfg, &state), Some(1)); // battery_stop
    }

    #[test]
    fn ac_battery_on_ac() {
        let cfg = make_config(AutoMode::AcBattery, 30);
        let state = UPowerState { on_battery: false, battery_percentage: Some(80) };
        assert_eq!(compute_desired_stop(&cfg, &state), Some(4)); // ac_default_stop
    }

    #[test]
    fn battery_pct_below_threshold() {
        let cfg = make_config(AutoMode::BatteryPct, 30);
        let state = UPowerState { on_battery: true, battery_percentage: Some(20) };
        assert_eq!(compute_desired_stop(&cfg, &state), Some(1));
    }

    #[test]
    fn battery_pct_above_threshold() {
        let cfg = make_config(AutoMode::BatteryPct, 30);
        let state = UPowerState { on_battery: false, battery_percentage: Some(50) };
        assert_eq!(compute_desired_stop(&cfg, &state), Some(4));
    }

    #[test]
    fn battery_pct_at_threshold() {
        let cfg = make_config(AutoMode::BatteryPct, 30);
        let state = UPowerState { on_battery: true, battery_percentage: Some(30) };
        assert_eq!(compute_desired_stop(&cfg, &state), Some(4)); // 30 < 30 is false → ac_default
    }

    #[test]
    fn battery_pct_no_battery_defaults_to_ac() {
        let cfg = make_config(AutoMode::BatteryPct, 30);
        let state = UPowerState { on_battery: false, battery_percentage: None };
        assert_eq!(compute_desired_stop(&cfg, &state), Some(4)); // 100 >= 30 → ac_default
    }
}
