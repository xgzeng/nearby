use anyhow::Result;
use crate::config::{save_config, backup_config, default_config_dir, ConfigData, Connection, BLEConnection, Action, ProximityAction};
use crate::commands::Command;
use std::fs;
use std::path::PathBuf;
use std::io::{self};
use std::time::Duration;

use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Clear},
    Terminal,
};

pub mod discovery;

/// Entry point for the TUI setup wizard using Ratatui with multi-device support.
pub async fn run_wizard(config_path: Option<PathBuf>) -> Result<()> {
    // 1. Prepare configuration path and load initial data
    let config_path = config_path.unwrap_or_else(|| default_config_dir().join("config.toml"));
    let initial_data = if config_path.exists() {
        let content = fs::read_to_string(&config_path)?;
        toml::from_str::<ConfigData>(&content).unwrap_or_default()
    } else {
        ConfigData::default()
    };

    let session = bluer::Session::new().await?;
    let adapter = session.default_adapter().await?;
    adapter.set_powered(true).await?;

    // 2. Multi-device Setup UI
    let (final_configs, final_seat) = discovery::setup_devices(&adapter, &initial_data, &config_path).await?;
    
    if final_configs.is_empty() {
        println!("No devices configured. Configuration will be empty.");
    }

    // 3. Confirmation Screen (Popup)
    enable_raw_mode()?;
    execute!(io::stdout(), EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;
    
    let mut confirmed = None;
    while confirmed.is_none() {
        terminal.draw(|f| {
            let area = f.area();
            
            // Background is just black or slightly dimmed
            let background = Block::default().bg(Color::Reset);
            f.render_widget(background, area);

            let popup_area = centered_rect(60, 25, area);
            f.render_widget(Clear, popup_area); // This clears the area for the popup

            let warning_text = vec![
                Line::from(""),
                Line::from(Span::styled(" SECURITY WARNING ", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))),
                Line::from(""),
                Line::from("Enabling automatic 'Unlock' can allow anyone with your device"),
                Line::from("to access your session. This is a potential security risk."),
                Line::from(""),
                Line::from("Are you sure you want to save these changes?"),
                Line::from(""),
                Line::from(vec![
                    Span::raw("Press "),
                    Span::styled("y", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                    Span::raw(" to Save, or "),
                    Span::styled("n", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                    Span::raw(" to Abort"),
                ]),
            ];

            let p = Paragraph::new(warning_text)
                .alignment(ratatui::layout::Alignment::Center)
                .block(Block::default()
                    .borders(Borders::ALL)
                    .title(" Confirmation ")
                    .border_style(Style::default().fg(Color::Red)));
            
            f.render_widget(p, popup_area);
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
    let mut new_data = ConfigData { connection: vec![], seat: final_seat };
    for (dev, n_rssi, a_rssi) in final_configs {
        new_data.connection.push(Connection::Ble(BLEConnection {
            mac: dev.addr.clone(),
            name: dev.name.clone(),
            rssi: None,
            actions: Some(vec![
                Action::Nearby(ProximityAction { threshold: n_rssi, command: Command::Unlock }),
                Action::Away(ProximityAction { threshold: a_rssi, command: Command::Lock }),
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

/// helper function to create a centered rect using up certain percentage of available rect `r`
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
