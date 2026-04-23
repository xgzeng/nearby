use anyhow::{Context, Result};
use bluer::{AdapterEvent, DeviceEvent, DeviceProperty, DiscoveryFilter, DiscoveryTransport};
use clap::{Parser, Subcommand};
use config::get_config;
use futures::{pin_mut, stream::SelectAll, StreamExt};
use std::{
    path::PathBuf,
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tokio::{spawn, time::sleep};

mod commands;
mod config;
mod idle;
mod setup;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Sets a custom config file
    #[arg(short, long, value_name = "CONFIG_FILE")]
    config: Option<PathBuf>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Run the interactive setup wizard
    Setup,
}

async fn check_bluetooth_permissions() -> Result<()> {
    match bluer::Session::new().await {
        Ok(_) => Ok(()),
        Err(e) => {
            if e.kind == bluer::ErrorKind::NotAuthorized {
                anyhow::bail!(
                    "Bluetooth access denied. Please ensure your user is in the 'bluetooth' group.\n\
                    You can add your user with: sudo usermod -aG bluetooth $USER\n\
                    Note: You may need to log out and back in for changes to take effect."
                );
            }
            Err(e.into())
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Setup) => {
            check_bluetooth_permissions().await?;
            setup::run_wizard(cli.config).await?;
        }
        None => {
            check_bluetooth_permissions().await?;
            run_daemon(cli.config).await?;
        }
    }

    Ok(())
}

async fn run_daemon(config_path: Option<PathBuf>) -> Result<()> {
    log::info!("Starting daemon...");
    let config = Arc::new(get_config(config_path.as_deref())?);
    if config.is_empty() {
        log::error!("No connections configured. Run 'nearby setup' or provide a config file.");
        return Ok(());
    }

    let login_manager = idle::login_manager().await?;

    let session = bluer::Session::new().await?;
    let adapter = session.default_adapter().await?;
    adapter.set_powered(true).await?;

    let filter = DiscoveryFilter {
        transport: DiscoveryTransport::Le,
        ..Default::default()
    };
    adapter.set_discovery_filter(filter).await?;
    let device_events = adapter.discover_devices_with_changes().await?;
    pin_mut!(device_events);

    let mut prop_change_events = SelectAll::new();

    let cfg = config.clone();
    spawn(async move {
        loop {
            sleep(std::time::Duration::from_secs(1)).await;

            let can_unlock = cfg.can_unlock();
            if can_unlock {
                log::info!("Unlocking...");
                if let Err(e) = login_manager.unlock_sessions().await {
                    log::error!("Failed to unlock sessions: {}", e);
                }
                continue;
            }

            let should_lock = cfg.should_lock();
            if should_lock {
                log::info!("Locking...");
                if let Err(e) = login_manager.lock_sessions().await {
                    log::error!("Failed to lock sessions: {}", e);
                }
                continue;
            }

            let keep_unlocked = cfg.keep_unlocked();
            if keep_unlocked {
                continue;
            }

            if let Ok((idle, idle_since)) = idle::get_idle_hint().await {
                let idle_since = UNIX_EPOCH + Duration::from_micros(idle_since);
                let idle_for = SystemTime::now()
                    .duration_since(idle_since)
                    .unwrap_or(Duration::from_secs(0));

                if idle && idle_for > Duration::from_secs(10) {
                    log::info!("Idle for: {:?}", idle_for);
                    if let Err(e) = login_manager.lock_sessions().await {
                        log::error!("Failed to lock sessions: {}", e);
                    }
                }
            }
        }
    });

    loop {
        tokio::select! {
            Some(device_event) = device_events.next() => {
                match device_event {
                    AdapterEvent::DeviceAdded(addr) => {
                        if !config.contains(&addr.to_string()) {
                            log::debug!("Device Added: {:?}", addr);
                            continue;
                        }

                        let device = adapter.device(addr).context("Failed to get device")?;
                        let rssi = device.rssi().await.context("Failed to get RSSI")?;
                        let name = device.name().await.context("Failed to get device name")?;
                        log::info!("Device Added: {:?}({:?}), {:?} dBm", addr, name, rssi);

                        config.update_rssi(&addr.to_string(), rssi);

                        // watch for device changes
                        let change_events = device.events().await?.map(move |evt| (addr, evt));
                        prop_change_events.push(change_events);
                    }
                    AdapterEvent::DeviceRemoved(addr) => {
                        if !config.contains(&addr.to_string()) {
                            log::debug!("Device Removed: {:?}", addr);
                            continue;
                        }
                        log::info!("Device Removed: {:?}", addr);
                        config.update_rssi(&addr.to_string(), None);
                    }
                    _ => (),
                }
            }
            Some((addr, DeviceEvent::PropertyChanged(property))) = prop_change_events.next() => {
                match property {
                    DeviceProperty::Rssi(rssi) => {
                        config.update_rssi(&addr.to_string(), Some(rssi));
                        log::info!("RSSI changed: {} {}dBm (~{:.2}m)", addr, rssi, distance_rssi(rssi));
                    },
                    _ => {
                        log::debug!("Property changed: {} {:?}", addr, property);
                    }
                }
            }
        }
    }
}

pub fn distance_rssi(rssi: i16) -> f32 {
    // 10 ^ ((Measured Power - RSSI) / (10 * N))
    // Measured Power is -69 (at 1m), N is 2 (environmental factor)
    let exponent = (-69 - rssi) as f32 / 20.0;
    10_f32.powf(exponent)
}
