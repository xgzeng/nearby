use inquire::Text;

pub fn run_wizard() -> anyhow::Result<()> {
    println!("Welcome to nearby setup wizard!");
    println!("This tool will help you configure nearby to lock and unlock your session based on Bluetooth device proximity.");
    
    let _ = Text::new("Press Enter to start...")
        .with_help_message("Press any key to continue")
        .prompt_skippable()?;

    Ok(())
}
