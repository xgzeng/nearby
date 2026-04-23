use anyhow::Result;
use bluer::{Adapter, AdapterEvent, DiscoveryFilter, DiscoveryTransport};
use futures::stream::SelectAll;
use futures::StreamExt;
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::sleep;

use crossterm::cursor::{Hide, Show};
use crossterm::event::{self, Event, KeyCode};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use std::io::{self};
use std::path::Path;

use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState},
    Terminal,
};

use crate::config::ConfigData;

#[derive(Debug, Clone)]
pub struct DiscoveredDevice {
    pub name: Option<String>,
    pub addr: String,
    pub rssi: Option<i16>,
}

#[derive(PartialEq)]
enum FilterField {
    None,
    Mac,
    Rssi,
}

/// Unified UI showing device list as a table with multi-device calibration and removal.
pub async fn setup_devices(
    adapter: &Adapter,
    initial_config: &ConfigData,
    config_path: &Path,
) -> Result<Vec<(DiscoveredDevice, i16, i16)>> {
    let filter = DiscoveryFilter {
        transport: DiscoveryTransport::Le,
        duplicate_data: true,
        ..Default::default()
    };
    adapter.set_discovery_filter(filter.clone()).await?;

    let mut device_events = adapter.discover_devices_with_changes().await?;
    let mut devices_map: HashMap<String, DiscoveredDevice> = HashMap::new();
    let mut table_state = TableState::default();
    table_state.select(Some(0));

    let mut all_change_events = SelectAll::new();

    // Filter state
    let mut show_only_named = false;
    let mut mac_filter = String::new();
    let mut rssi_filter: Option<i16> = None;
    let mut active_field = FilterField::None;

    // Calibration state (mapped by address)
    let mut thresholds: HashMap<String, (Option<i16>, Option<i16>)> = HashMap::new();

    // Load initial config into devices and thresholds
    for conn in &initial_config.connection {
        if let Some(ble) = conn.get_ble() {
            devices_map.insert(
                ble.mac.clone(),
                DiscoveredDevice {
                    name: ble.name.clone(),
                    addr: ble.mac.clone(),
                    rssi: None, // Placeholder for offline
                },
            );

            if let Some(actions) = &ble.actions {
                let mut n = None;
                let mut a = None;
                for action in actions {
                    match action {
                        crate::config::Action::Nearby(p) => n = Some(p.threshold),
                        crate::config::Action::Away(p) => a = Some(p.threshold),
                    }
                }
                thresholds.insert(ble.mac.clone(), (n, a));
            }
        }
    }

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, Hide)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = 'main: loop {
        // 1. Filter and Sort
        let mut devices_list: Vec<DiscoveredDevice> = devices_map
            .values()
            .filter(|d| {
                if show_only_named && d.name.is_none() {
                    return false;
                }
                if !mac_filter.is_empty()
                    && !d.addr.to_lowercase().contains(&mac_filter.to_lowercase())
                {
                    return false;
                }
                if let Some(t) = rssi_filter {
                    if let Some(rssi) = d.rssi {
                        if rssi < t {
                            return false;
                        }
                    } else {
                        return false;
                    }
                }
                true
            })
            .cloned()
            .collect();
        devices_list.sort_by(|a, b| b.rssi.cmp(&a.rssi));

        if let Some(selected) = table_state.selected() {
            if selected >= devices_list.len() && !devices_list.is_empty() {
                table_state.select(Some(devices_list.len() - 1));
            }
        } else if !devices_list.is_empty() {
            table_state.select(Some(0));
        }

        let highlighted_device = table_state.selected().and_then(|i| devices_list.get(i));

        // 2. Render
        terminal.draw(|f| {
            let main_chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([
                    Constraint::Length(3), // Title
                    Constraint::Length(3), // Filters
                    Constraint::Min(0),    // Table
                    Constraint::Length(4), // Help
                ])
                .split(f.area());

            // Title
            let title_text = format!(" Nearby BLE Setup (Config: {}) ", config_path.display());
            f.render_widget(
                Paragraph::new(title_text.bold().cyan()).block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::Cyan)),
                ),
                main_chunks[0],
            );

            // Filters
            let named_color = if show_only_named {
                Color::Green
            } else {
                Color::DarkGray
            };
            let mac_color = if active_field == FilterField::Mac {
                Color::Yellow
            } else {
                Color::Gray
            };
            let rssi_color = if active_field == FilterField::Rssi {
                Color::Yellow
            } else {
                Color::Gray
            };

            let rssi_val_str = rssi_filter
                .map(|v| v.to_string())
                .unwrap_or_else(|| "None".to_string());
            let filter_line = Line::from(vec![
                Span::raw(" [f] Named Only: "),
                Span::styled(
                    if show_only_named { "ON " } else { "OFF" },
                    Style::default().fg(named_color),
                ),
                Span::raw(" | [m] MAC: "),
                Span::styled(
                    if active_field == FilterField::Mac {
                        format!("{}█", mac_filter)
                    } else {
                        mac_filter.clone()
                    },
                    Style::default().fg(mac_color),
                ),
                Span::raw(" | [r] RSSI Min: "),
                Span::styled(
                    if active_field == FilterField::Rssi {
                        format!("{}█", rssi_val_str)
                    } else {
                        rssi_val_str
                    },
                    Style::default().fg(rssi_color),
                ),
                Span::raw(" | [c] Clear Filters"),
            ]);
            f.render_widget(
                Paragraph::new(filter_line)
                    .block(Block::default().borders(Borders::ALL).title(" Filters ")),
                main_chunks[1],
            );

            // Table
            let header_cells = ["RSSI", "Name", "MAC Address", "Nearby", "Away"]
                .iter()
                .map(|h| {
                    Cell::from(*h).style(
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    )
                });
            let header = Row::new(header_cells).height(1).bottom_margin(1);

            let rows = devices_list.iter().enumerate().map(|(_, d)| {
                let rssi_display = if let Some(rssi) = d.rssi {
                    format!("{} dBm", rssi)
                } else {
                    "OFFLINE".to_string()
                };
                let rssi_style = if let Some(rssi) = d.rssi {
                    if rssi > -60 {
                        Color::Green
                    } else if rssi > -80 {
                        Color::Yellow
                    } else {
                        Color::Red
                    }
                } else {
                    Color::DarkGray
                };
                let (n_set, a_set) = thresholds.get(&d.addr).cloned().unwrap_or((None, None));

                let is_configured = n_set.is_some() && a_set.is_some();
                let name_style = if is_configured {
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };

                let name_display = d.name.as_deref().unwrap_or("<Unknown>");

                Row::new(vec![
                    Cell::from(rssi_display).style(Style::default().fg(rssi_style)),
                    Cell::from(name_display).style(name_style),
                    Cell::from(d.addr.clone()).style(Style::default().fg(Color::DarkGray)),
                    Cell::from(
                        n_set
                            .map(|v| format!("{} dBm", v))
                            .unwrap_or_else(|| "--".into()),
                    )
                    .style(Style::default().fg(Color::Green)),
                    Cell::from(
                        a_set
                            .map(|v| format!("{} dBm", v))
                            .unwrap_or_else(|| "--".into()),
                    )
                    .style(Style::default().fg(Color::Red)),
                ])
            });

            let table = Table::new(
                rows,
                [
                    Constraint::Length(10),
                    Constraint::Max(32),
                    Constraint::Length(20),
                    Constraint::Length(12),
                    Constraint::Length(12),
                ],
            )
            .header(header)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Discovered & Configured Devices "),
            )
            .row_highlight_style(
                Style::default()
                    .bg(Color::Indexed(236))
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(">");

            f.render_stateful_widget(table, main_chunks[2], &mut table_state);

            // Help
            let help_text = vec![
                Line::from(
                    "↑/↓: Navigate | f/m/r: Filters | c: Clear Filters | Space: Toggle Config",
                ),
                Line::from(vec![
                    Span::styled(
                        "n",
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(": Set Nearby | "),
                    Span::styled(
                        "a",
                        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(": Set Away | "),
                    Span::styled(
                        "Enter",
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(": Save All & Finish"),
                ]),
            ];
            f.render_widget(
                Paragraph::new(help_text).block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(" Help ")
                        .border_style(Style::default().fg(Color::DarkGray)),
                ),
                main_chunks[3],
            );
        })?;

        // 3. Events
        tokio::select! {
            _ = sleep(Duration::from_millis(50)) => {
                if event::poll(Duration::from_millis(0))? {
                    if let Event::Key(key) = event::read()? {
                        if active_field != FilterField::None {
                            match key.code {
                                KeyCode::Char(c) => {
                                    if active_field == FilterField::Mac { mac_filter.push(c); }
                                    else if active_field == FilterField::Rssi && (c.is_digit(10) || c == '-') {
                                        let mut s = rssi_filter.map(|v| v.to_string()).unwrap_or_default();
                                        s.push(c);
                                        if let Ok(v) = s.parse::<i16>() { rssi_filter = Some(v); }
                                    }
                                }
                                KeyCode::Backspace => {
                                    if active_field == FilterField::Mac { mac_filter.pop(); }
                                    else if active_field == FilterField::Rssi {
                                        let mut s = rssi_filter.map(|v| v.to_string()).unwrap_or_default();
                                        s.pop();
                                        rssi_filter = s.parse::<i16>().ok();
                                    }
                                }
                                KeyCode::Enter | KeyCode::Esc => { active_field = FilterField::None; }
                                _ => {}
                            }
                            continue 'main;
                        }

                        match key.code {
                            KeyCode::Up => {
                                let i = table_state.selected().map(|i| if i > 0 { i - 1 } else { 0 }).unwrap_or(0);
                                table_state.select(Some(i));
                            }
                            KeyCode::Down => {
                                let i = table_state.selected().map(|i| if i < devices_list.len() - 1 { i + 1 } else { i }).unwrap_or(0);
                                table_state.select(Some(i));
                            }
                            KeyCode::Char('f') => show_only_named = !show_only_named,
                            KeyCode::Char('m') => active_field = FilterField::Mac,
                            KeyCode::Char('r') => { active_field = FilterField::Rssi; if rssi_filter.is_none() { rssi_filter = Some(-100); } },
                            KeyCode::Char('c') => { show_only_named = false; mac_filter.clear(); rssi_filter = None; }
                            KeyCode::Char('n') => {
                                if let Some(d) = highlighted_device {
                                    if let Some(rssi) = d.rssi { // Only set if online
                                        let entry = thresholds.entry(d.addr.clone()).or_insert((None, None));
                                        entry.0 = Some(rssi);
                                    }
                                }
                            }
                            KeyCode::Char('a') => {
                                if let Some(d) = highlighted_device {
                                    if let Some(rssi) = d.rssi { // Only set if online
                                        let entry = thresholds.entry(d.addr.clone()).or_insert((None, None));
                                        entry.1 = Some(rssi);
                                    }
                                }
                            }
                            KeyCode::Char(' ') => {
                                if let Some(d) = highlighted_device {
                                    if thresholds.contains_key(&d.addr) {
                                        thresholds.remove(&d.addr);
                                    } else if let Some(rssi) = d.rssi {
                                        thresholds.insert(d.addr.clone(), (Some(rssi - 3), Some(rssi - 6)));
                                    }
                                }
                            }
                            KeyCode::Char('d') | KeyCode::Delete => {
                                if let Some(d) = highlighted_device {
                                    thresholds.remove(&d.addr);
                                }
                            }
                            KeyCode::Enter => {
                                // Finalize: collect all devices that have both thresholds set
                                let mut final_list = Vec::new();
                                for (addr, (n, a)) in &thresholds {
                                    if let (Some(nearby), Some(away)) = (*n, *a) {
                                        // Find device info from our map
                                        if let Some(dev) = devices_map.get(addr) {
                                            final_list.push((dev.clone(), nearby, away));
                                        }
                                    }
                                }
                                break 'main Ok(final_list);
                            }
                            KeyCode::Char('q') | KeyCode::Esc => break 'main Err(anyhow::anyhow!("Setup cancelled.")),
                            _ => {}
                        }
                    }
                }
            }
            Some(event) = device_events.next() => {
                match event {
                    AdapterEvent::DeviceAdded(addr) => {
                        let device = adapter.device(addr)?;
                        let name = device.name().await?;
                        let rssi = device.rssi().await?;
                        devices_map.insert(addr.to_string(), DiscoveredDevice { name: name.clone(), addr: addr.to_string(), rssi });

                        // Watch for property changes
                        if let Ok(events) = device.events().await {
                            all_change_events.push(events.map(move |e| (addr, e)));
                        }
                    }
                    AdapterEvent::DeviceRemoved(addr) => {
                        let addr_str = addr.to_string();
                        if thresholds.contains_key(&addr_str) {
                            if let Some(device) = devices_map.get_mut(&addr_str) {
                                device.rssi = None;
                            }
                        } else {
                            devices_map.remove(&addr_str);
                        }
                    }
                    AdapterEvent::PropertyChanged(bluer::AdapterProperty::Discovering(false)) => {
                        adapter.set_discovery_filter(filter.clone()).await?;
                    }
                    _ => {}
                }
            }
            Some((addr, bluer::DeviceEvent::PropertyChanged(property))) = all_change_events.next() => {
                match property {
                    bluer::DeviceProperty::Name(name) => {
                        if let Some(device) = devices_map.get_mut(&addr.to_string()) {
                            device.name = Some(name);
                        }
                    }
                    bluer::DeviceProperty::Rssi(rssi) => {
                        if let Some(device) = devices_map.get_mut(&addr.to_string()) {
                            device.rssi = Some(rssi);
                        }
                    }
                    _ => {}
                }
            }
        }
    };

    execute!(terminal.backend_mut(), Show, LeaveAlternateScreen)?;
    disable_raw_mode()?;
    result
}
