use bluer::{
    AdapterEvent, Address, DeviceEvent, DeviceProperty, DiscoveryFilter, DiscoveryTransport,
};
use clap::Parser;
use commands::run;
use config::get_config;
use futures::{pin_mut, stream::SelectAll, StreamExt};
use std::{
    collections::HashSet,
    path::PathBuf,
    sync::{Arc, Mutex},
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
    let cli = Cli::parse();
    let config = Arc::new(Mutex::new(get_config(cli.config.as_deref())?));
    if config.lock().unwrap().is_empty() {
        println!("No connections configured. Exiting.");
        return Ok(());
    }

    let ble_addresses: HashSet<_> = config
        .lock()
        .unwrap()
        .connections()
        .iter()
        .filter_map(|c| c.get_ble())
        .map(|c| c.mac.parse::<Address>().unwrap())
        .collect();

    env_logger::init();
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

            let can_unlock = cfg.lock().unwrap().can_unlock();
            if can_unlock {
                println!("Unlocking...");
                run("sudo loginctl unlock-sessions").unwrap();
                continue;
            }

            let should_lock = cfg.lock().unwrap().should_lock();
            if !should_lock {
                continue;
            }

            let keep_unlocked = cfg.lock().unwrap().keep_unlocked();
            if keep_unlocked {
                continue;
            }

            let (idle, idle_since) = idle::get_idle_hint().await.unwrap();
            let idle_since = UNIX_EPOCH + Duration::from_micros(idle_since);
            let idle_for = SystemTime::now().duration_since(idle_since).unwrap();

            if idle && idle_for > Duration::from_secs(10) {
                println!("Idle for: {:?}", idle_for);
                run("sudo loginctl lock-sessions").unwrap();
            }
        }
    });

    loop {
        tokio::select! {
            Some(device_event) = device_events.next() => {
                match device_event {
                    AdapterEvent::DeviceAdded(addr) => {
                        if !ble_addresses.is_empty() && !ble_addresses.contains(&addr) {
                            continue;
                        }
                        let device = adapter.device(addr)?;
                        let rssi = device.rssi().await?.unwrap_or_default();

                        config.lock().unwrap().update_rssi(&addr.to_string(), rssi);
                        println!("{:?} {:.2}",addr,distance_rssi(rssi));

                        // with changes
                        let device = adapter.device(addr)?;
                        let change_events = device.events().await?.map(move |evt| (addr, evt));
                        all_change_events.push(change_events);
                    }
                    AdapterEvent::DeviceRemoved(addr) => {
                        println!("{addr} Removed");
                        config.lock().unwrap().update_rssi(&addr.to_string(), -99);
                    }
                    _ => (),
                }
            }
            Some((addr, DeviceEvent::PropertyChanged(property))) = all_change_events.next() => {
                match property {
                    DeviceProperty::Rssi(rssi) => {
                        config.lock().unwrap().update_rssi(&addr.to_string(), rssi);
                        println!("{:?} {:.2}m", addr, distance_rssi(rssi));
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
