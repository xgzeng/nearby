use bluer::{
    AdapterEvent, DeviceEvent, DeviceProperty, DiscoveryFilter, DiscoveryTransport,
};
use clap::Parser;
use commands::run;
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

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Sets a custom config file
    #[arg(short, long, value_name = "CONFIG_FILE")]
    config: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    let cli = Cli::parse();
    let config = Arc::new(get_config(cli.config.as_deref())?);
    if config.is_empty() {
        log::error!("No connections configured. Exiting.");
        return Ok(());
    }

    let session = bluer::Session::new().await?;

    let adapter = session.default_adapter().await?;
    adapter.set_powered(true).await?;

    let filter = DiscoveryFilter {
        transport: DiscoveryTransport::Le,
        duplicate_data: true,
        ..Default::default()
    };
    adapter.set_discovery_filter(filter).await?;
    let device_events = adapter.discover_devices().await?;
    pin_mut!(device_events);

    let mut all_change_events = SelectAll::new();

    let cfg = config.clone();
    spawn(async move {
        loop {
            sleep(std::time::Duration::from_secs(1)).await;

            let can_unlock = cfg.can_unlock();
            if can_unlock {
                log::info!("Unlocking...");
                run("sudo loginctl unlock-sessions").unwrap();
                continue;
            }

            let should_lock = cfg.should_lock();
            if !should_lock {
                continue;
            }

            let keep_unlocked = cfg.keep_unlocked();
            if keep_unlocked {
                continue;
            }

            let (idle, idle_since) = idle::get_idle_hint().await.unwrap();
            let idle_since = UNIX_EPOCH + Duration::from_micros(idle_since);
            let idle_for = SystemTime::now().duration_since(idle_since).unwrap();

            if idle && idle_for > Duration::from_secs(10) {
                log::info!("Idle for: {:?}", idle_for);
                run("sudo loginctl lock-sessions").unwrap();
            }
        }
    });

    loop {
        tokio::select! {
            Some(device_event) = device_events.next() => {
                match device_event {
                    AdapterEvent::DeviceAdded(addr) => {
                        log::debug!("{} Added", addr);

                        if !config.contains(&addr.to_string()) {
                            continue;
                        }
                        let device = adapter.device(addr)?;
                        let rssi = device.rssi().await?.unwrap_or_default();

                        config.update_rssi(&addr.to_string(), rssi);
                        log::info!("{:?} {:.2}",addr,distance_rssi(rssi));

                        // with changes
                        // let device = adapter.device(addr)?;
                        let change_events = device.events().await?.map(move |evt| (addr, evt));
                        all_change_events.push(change_events);
                    }
                    AdapterEvent::DeviceRemoved(addr) => {
                        log::debug!("{} Removed", addr);

                        if !config.contains(&addr.to_string()) {
                            continue;
                        }

                        config.update_rssi(&addr.to_string(), -99);
                    }
                    _ => (),
                }
            }
            Some((addr, DeviceEvent::PropertyChanged(property))) = all_change_events.next() => {
                match property {
                    DeviceProperty::Rssi(rssi) => {
                        config.update_rssi(&addr.to_string(), rssi);
                        log::info!("{:?} {:.2}m", addr, distance_rssi(rssi));
                    },
                    _ => {
                        // println!("    {property:?}");
                    }
                }
            }
            else => break
        }
    }

    Ok(())
}

pub fn distance_rssi(rssi: i16) -> f32 {
    // 10 ^ ((-69 – (-60))/(10 * 2))
    let exponent = (-69 - rssi) as f32 / (10_i16.pow(2)) as f32;
    10_f32.powf(exponent)
}
