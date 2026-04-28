use anyhow::Result;
use futures_lite::stream::StreamExt;
use tokio::sync::mpsc::Sender;
use zbus::{proxy, Connection};
use zbus::fdo::PropertiesProxy;

#[proxy(
    interface = "org.freedesktop.UPower",
    default_service = "org.freedesktop.UPower",
    default_path = "/org/freedesktop/UPower"
)]
trait UPower {
    #[zbus(property)]
    fn on_battery(&self) -> zbus::Result<bool>;

    fn enumerate_devices(&self) -> zbus::Result<Vec<zbus::zvariant::OwnedObjectPath>>;
}

#[proxy(
    interface = "org.freedesktop.UPower.Device",
    default_service = "org.freedesktop.UPower"
)]
trait UPowerDevice {
    #[zbus(property)]
    fn type_(&self) -> zbus::Result<u32>;

    #[zbus(property)]
    fn percentage(&self) -> zbus::Result<f64>;
}

pub struct UPowerState {
    pub on_battery: bool,
    pub battery_percentage: Option<u8>,
}

pub async fn get_state(conn: &Connection) -> Result<UPowerState> {
    let upower = UPowerProxy::new(conn).await?;
    let on_battery = upower.on_battery().await?;

    let devices = upower.enumerate_devices().await?;
    let mut battery_percentage: Option<u8> = None;

    for path in devices {
        let dev = UPowerDeviceProxy::builder(conn)
            .path(path)?
            .build()
            .await?;
        if dev.type_().await? == 2 {
            let pct = dev.percentage().await?;
            battery_percentage = Some(pct.clamp(0.0, 100.0) as u8);
            break;
        }
    }

    Ok(UPowerState { on_battery, battery_percentage })
}

pub async fn watch_changes(conn: &Connection, tx: Sender<UPowerState>) -> Result<()> {
    let props_root = PropertiesProxy::builder(conn)
        .destination("org.freedesktop.UPower")?
        .path("/org/freedesktop/UPower")?
        .build()
        .await?;

    let mut stream_root = props_root.receive_properties_changed().await?;

    // Discover battery device path for per-device PropertiesChanged subscription.
    let battery_path: Option<zbus::zvariant::OwnedObjectPath> = {
        let upower = UPowerProxy::new(conn).await?;
        let devices = upower.enumerate_devices().await?;
        let mut found = None;
        for path in devices {
            let dev = UPowerDeviceProxy::builder(conn)
                .path(path.clone())?
                .build()
                .await?;
            if dev.type_().await? == 2 {
                found = Some(path);
                break;
            }
        }
        found
    };

    // Subscribe to battery device PropertiesChanged if a battery was found.
    if let Some(bat_path) = battery_path {
        let props_bat = PropertiesProxy::builder(conn)
            .destination("org.freedesktop.UPower")?
            .path(bat_path)?
            .build()
            .await?;
        let mut stream_bat = props_bat.receive_properties_changed().await?;

        loop {
            tokio::select! {
                msg = stream_root.next() => {
                    if msg.is_none() { break; }
                }
                msg = stream_bat.next() => {
                    if msg.is_none() { break; }
                }
            }
            match get_state(conn).await {
                Ok(state) => {
                    if tx.send(state).await.is_err() {
                        break;
                    }
                }
                Err(e) => {
                    tracing::warn!("upower get_state error: {e}");
                }
            }
        }
    } else {
        // No battery found (desktop): watch only UPower root.
        loop {
            if stream_root.next().await.is_none() {
                break;
            }
            match get_state(conn).await {
                Ok(state) => {
                    if tx.send(state).await.is_err() {
                        break;
                    }
                }
                Err(e) => {
                    tracing::warn!("upower get_state error: {e}");
                }
            }
        }
    }

    Ok(())
}
