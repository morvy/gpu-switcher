use std::sync::{Arc, Mutex};

use zbus::{fdo, interface, object_server::SignalContext};

use crate::config::{AutoMode, Config};
use crate::sysfs::AmdgpuNode;
use crate::{ppd, sysfs};

pub struct ManagerState {
    pub config: Config,
    pub amdgpu: AmdgpuNode,
}

pub struct Manager {
    pub state: Arc<Mutex<ManagerState>>,
    pub conn: zbus::Connection,
}

/// Apply a new stop: write sysfs, PPD, update config, emit signal.
/// Takes Arc to avoid holding MutexGuard across await points.
pub async fn apply_stop(
    state_arc: &Arc<Mutex<ManagerState>>,
    conn: &zbus::Connection,
    stop: u8,
    ctx: &SignalContext<'_>,
) -> fdo::Result<()> {
    // Sync sysfs write (no await needed).
    {
        let state = state_arc
            .lock()
            .map_err(|_| fdo::Error::Failed("state lock poisoned".into()))?;
        state
            .amdgpu
            .set_stop(stop)
            .map_err(|e| fdo::Error::Failed(e.to_string()))?;
    }

    // Async PPD call — no lock held.
    let ppd_profile =
        sysfs::stop_to_ppd(stop).map_err(|e| fdo::Error::Failed(e.to_string()))?;
    ppd::set_active_profile(conn, ppd_profile)
        .await
        .map_err(|e| fdo::Error::Failed(e.to_string()))?;

    // Update config and save (save is sync).
    {
        let mut state = state_arc
            .lock()
            .map_err(|_| fdo::Error::Failed("state lock poisoned".into()))?;
        state.config.profile.current_stop = stop;
        state
            .config
            .save()
            .map_err(|e| fdo::Error::Failed(e.to_string()))?;
    }

    Manager::stop_changed(ctx, stop).await?;

    Ok(())
}

#[interface(name = "net.gpuswitcher.Manager")]
impl Manager {
    async fn set_stop(
        &self,
        stop: u8,
        #[zbus(signal_context)] ctx: SignalContext<'_>,
    ) -> fdo::Result<()> {
        // TODO v2: call polkit to check net.gpuswitcher.set-profile for remote sessions.
        // Currently DBus conf restricts callers to at_console=true (Plasma session user),
        // which is sufficient for single-user desktop use.
        if !(1..=5).contains(&stop) {
            return Err(fdo::Error::InvalidArgs(format!(
                "stop must be 1–5, got {stop}"
            )));
        }
        apply_stop(&self.state, &self.conn, stop, &ctx).await
    }

    async fn set_auto_mode(
        &self,
        mode: String,
        threshold: u8,
        #[zbus(signal_context)] ctx: SignalContext<'_>,
    ) -> fdo::Result<()> {
        let parsed = match mode.as_str() {
            "manual" => AutoMode::Manual,
            "ac_battery" => AutoMode::AcBattery,
            "battery_pct" => AutoMode::BatteryPct,
            other => {
                return Err(fdo::Error::InvalidArgs(format!(
                    "unknown mode {other:?}; expected manual, ac_battery, or battery_pct"
                )));
            }
        };

        let (new_config, current_stop) = {
            let mut state = self
                .state
                .lock()
                .map_err(|_| fdo::Error::Failed("state lock poisoned".into()))?;
            state.config.auto.mode = parsed;
            state.config.auto.battery_threshold = threshold;
            state
                .config
                .save()
                .map_err(|e| fdo::Error::Failed(e.to_string()))?;
            (state.config.clone(), state.config.profile.current_stop)
        }; // guard explicitly dropped here

        // Trigger immediate apply so the new rule takes effect without waiting for
        // the next UPower event.
        let upower_state = crate::upower::get_state(&self.conn)
            .await
            .map_err(|e| fdo::Error::Failed(e.to_string()))?;
        if let Some(desired) = crate::automode::compute_desired_stop(&new_config, &upower_state) {
            if desired != current_stop {
                apply_stop(&self.state, &self.conn, desired, &ctx).await?;
            }
        }

        Ok(())
    }

    async fn get_stop(&self) -> fdo::Result<u8> {
        let state = self
            .state
            .lock()
            .map_err(|_| fdo::Error::Failed("state lock poisoned".into()))?;
        Ok(state.config.profile.current_stop)
    }

    #[zbus(property)]
    async fn current_stop(&self) -> fdo::Result<u8> {
        let state = self
            .state
            .lock()
            .map_err(|_| fdo::Error::Failed("state lock poisoned".into()))?;
        Ok(state.config.profile.current_stop)
    }

    #[zbus(property)]
    async fn auto_mode(&self) -> fdo::Result<String> {
        let state = self
            .state
            .lock()
            .map_err(|_| fdo::Error::Failed("state lock poisoned".into()))?;
        let s = match state.config.auto.mode {
            AutoMode::Manual => "manual",
            AutoMode::AcBattery => "ac_battery",
            AutoMode::BatteryPct => "battery_pct",
        };
        Ok(s.to_string())
    }

    #[zbus(signal)]
    pub async fn stop_changed(ctx: &SignalContext<'_>, new_stop: u8) -> zbus::Result<()>;
}
