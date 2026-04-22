use anyhow::Result;
use crate::config::{save_config, backup_config, default_config_dir, ConfigData, Connection, BLEConnection, Action, ProximityAction};
use crate::commands::Command;
use crate::distance_rssi;
use std::fs;
use std::path::PathBuf;
use std::io::{self};

use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};

pub mod discovery;

/// Entry point for the TUI setup wizard using Ratatui with multi-device support.
pub async fn run_wizard(config_path: Option<PathBuf>) -> Result<()> {
    // 1. Prepare configuration path and load initial data
    let config_path = config_path.unwrap_or_else(|| default_config_dir().join("config.toml"));
    let initial_data = if config_path.exists() {
        let content = fs::read_to_string(&config_path)?;
        toml::from_str::<ConfigData>(&content).unwrap_or(ConfigData { connection: vec![] })
    } else {
        ConfigData { connection: vec![] }
    };

    let session = bluer::Session::new().await?;
    let adapter = session.default_adapter().await?;
    adapter.set_powered(true).await?;

    // 2. Multi-device Setup UI
    let final_configs = discovery::setup_devices(&adapter, &initial_data, &config_path).await?;
    
    if final_configs.is_empty() {
        println!("No devices configured. Configuration will be empty.");
    }

    // 3. Confirmation Screen
    enable_raw_mode()?;
    execute!(io::stdout(), EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;
    
    let mut confirmed = None;
    while confirmed.is_none() {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(2)
                .constraints([Constraint::Min(0), Constraint::Length(3)])
                .split(f.area());

            let mut confirm_text = vec![
                Line::from("Devices to be saved:".bold()),
                Line::from(""),
            ];

            if final_configs.is_empty() {
                confirm_text.push(Line::from("  None (All devices will be removed)".red()));
            } else {
                for (dev, n_rssi, a_rssi) in &final_configs {
                    confirm_text.push(Line::from(vec![
                        Span::raw("  • "),
                        Span::styled(format!("{} ({})", dev.name.as_deref().unwrap_or("<Unknown>"), dev.addr), Style::default().fg(Color::Cyan)),
                        Span::raw(format!(" | Nearby: {}dBm | Away: {}dBm", n_rssi, a_rssi)),
                    ]));
                }
            }

            confirm_text.push(Line::from(""));
            confirm_text.push(Line::from(vec![
                Span::styled("SECURITY WARNING: ", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                Span::raw("Enabling automatic 'Unlock' can allow anyone with your device to access your session."),
            ]));

            let p = Paragraph::new(confirm_text)
                .block(Block::default().borders(Borders::ALL).title(" Final Confirmation ").border_style(Style::default().fg(Color::Red)));
            f.render_widget(p, chunks[0]);

            let help = Line::from(vec![
                Span::raw("Press "),
                Span::styled("y", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                Span::raw(" to Save & Continue, or "),
                Span::styled("n", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                Span::raw(" to Abort"),
            ]);
            let help_p = Paragraph::new(help)
                .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::DarkGray)));
            f.render_widget(help_p, chunks[1]);
        })?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('y') => confirmed = Some(true),
                    KeyCode::Char('n') | KeyCode::Char('q') | KeyCode::Esc => confirmed = Some(false),
                    _ => {}
                }
            }
        }
    }

    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    disable_raw_mode()?;

    if !confirmed.unwrap() {
        println!("Aborting setup.");
        return Ok(());
    }

    // 4. Update and Save Configuration
    let mut new_data = ConfigData { connection: vec![] };
    for (dev, n_rssi, a_rssi) in final_configs {
        new_data.connection.push(Connection::Ble(BLEConnection {
            mac: dev.addr.clone(),
            name: dev.name.clone(),
            rssi: None,
            actions: Some(vec![
                Action::Nearby(ProximityAction { threshold: distance_rssi(n_rssi), command: Command::Unlock }),
                Action::Away(ProximityAction { threshold: distance_rssi(a_rssi), command: Command::Lock }),
            ]),
        }));
    }

    println!("\nBacking up current configuration...");
    backup_config(&config_path)?;
    println!("Saving configuration to {:?}...", config_path);
    save_config(&config_path, &new_data)?;
    println!("Configuration saved successfully! Restart the nearby service to apply changes.");
    
    Ok(())
}

use std::time::Duration;
