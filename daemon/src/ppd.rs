use anyhow::{bail, Result};
use zbus::proxy;

#[proxy(
    interface = "net.hadess.PowerProfiles",
    default_service = "net.hadess.PowerProfiles",
    default_path = "/net/hadess/PowerProfiles"
)]
trait PowerProfiles {
    #[zbus(property)]
    fn active_profile(&self) -> zbus::Result<String>;

    #[zbus(property)]
    fn set_active_profile(&self, profile: &str) -> zbus::Result<()>;
}

pub async fn set_active_profile(conn: &zbus::Connection, profile: &str) -> Result<()> {
    match profile {
        "power-saver" | "balanced" | "performance" => {}
        other => bail!("invalid power profile: {other:?}"),
    }

    let proxy = PowerProfilesProxy::new(conn).await?;
    proxy.set_active_profile(profile).await?;
    Ok(())
}
