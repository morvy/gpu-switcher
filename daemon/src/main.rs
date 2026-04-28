use anyhow::Result;
use clap::Parser;
use std::sync::{Arc, Mutex};
use zbus::ConnectionBuilder;

mod automode;
mod config;
mod dbus;
mod ppd;
mod sysfs;
mod upower;

use dbus::{Manager, ManagerState};

#[derive(Parser)]
#[command(author, version, about)]
struct Cli {
    #[command(subcommand)]
    cmd: Command,
}

#[derive(clap::Subcommand)]
enum Command {
    /// Run as system daemon (default when started by systemd)
    Daemon,
    /// Set profile stop immediately via DBus (1-5)
    Set { stop: u8 },
    /// Get current profile stop via DBus
    Get,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    match cli.cmd {
        Command::Daemon => run_daemon().await,
        Command::Set { stop } => {
            if !(1..=5).contains(&stop) {
                eprintln!("Error: stop must be 1-5 (1=Save, 2=Adaptive, 3=Standard, 4=Perf, 5=Max)");
                std::process::exit(1);
            }
            run_set(stop).await
        }
        Command::Get => run_get().await,
    }
}

async fn run_daemon() -> Result<()> {
    let config = config::Config::load()?;
    let amdgpu = sysfs::AmdgpuNode::discover()?;

    // Connection used for PPD calls — stored directly on Manager, outside the mutex.
    let ppd_conn = zbus::Connection::system().await?;

    let state_arc = Arc::new(Mutex::new(ManagerState {
        config,
        amdgpu,
    }));

    let manager = Manager {
        state: state_arc.clone(),
        conn: ppd_conn.clone(),
    };

    let serving_conn = ConnectionBuilder::system()?
        .serve_at("/net/gpuswitcher/Manager", manager)?
        .name("net.gpuswitcher.Manager")?
        .build()
        .await?;

    // Spawn UPower watcher task.
    let upower_conn = zbus::Connection::system().await?;
    let serving_conn_clone = serving_conn.clone();

    tokio::spawn(async move {
        let (tx, mut rx) = tokio::sync::mpsc::channel::<upower::UPowerState>(8);

        let watch_conn = upower_conn.clone();
        tokio::spawn(async move {
            if let Err(e) = upower::watch_changes(&watch_conn, tx).await {
                tracing::error!("upower watcher exited: {e}");
            }
        });

        while let Some(upower_state) = rx.recv().await {
            // Compute desired stop without holding the lock across await.
            let (desired, current) = {
                let state = match state_arc.lock() {
                    Ok(s) => s,
                    Err(e) => {
                        tracing::error!("state lock poisoned: {e}");
                        break;
                    }
                };
                let desired = automode::compute_desired_stop(&state.config, &upower_state);
                let current = state.config.profile.current_stop;
                (desired, current)
            };

            if let Some(new_stop) = desired {
                if new_stop != current {
                    match serving_conn_clone
                        .object_server()
                        .interface::<_, Manager>("/net/gpuswitcher/Manager")
                        .await
                    {
                        Ok(iface_ref) => {
                            let ctx = iface_ref.signal_context().clone();
                            if let Err(e) =
                                dbus::apply_stop(&state_arc, &ppd_conn, new_stop, &ctx).await
                            {
                                tracing::error!("auto apply_stop failed: {e}");
                            }
                        }
                        Err(e) => {
                            tracing::error!("failed to get interface ref: {e}");
                        }
                    }
                }
            }
        }
    });

    tracing::info!("gpu-switcher-daemon running");
    std::future::pending::<()>().await;
    Ok(())
}

async fn run_set(stop: u8) -> Result<()> {
    let conn = zbus::Connection::system().await?;
    let proxy = zbus::Proxy::new(
        &conn,
        "net.gpuswitcher.Manager",
        "/net/gpuswitcher/Manager",
        "net.gpuswitcher.Manager",
    )
    .await?;
    proxy.call_method("SetStop", &(stop,)).await?;
    println!("Stop set to {stop}");
    Ok(())
}

async fn run_get() -> Result<()> {
    let conn = zbus::Connection::system().await?;
    let proxy = zbus::Proxy::new(
        &conn,
        "net.gpuswitcher.Manager",
        "/net/gpuswitcher/Manager",
        "net.gpuswitcher.Manager",
    )
    .await?;
    let stop: u8 = proxy
        .call_method("GetStop", &())
        .await?
        .body()
        .deserialize()?;
    println!("{stop}");
    Ok(())
}
