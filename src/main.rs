mod config;
mod tui;
mod validation;
mod state;

use config::Config;
use tui::run_tui;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load config (from file or default)
    let config = Config::load().unwrap_or_else(|e| {
        eprintln!("Warning: {}", e);
        Config::default()
    });

    run_tui(config)?;

    Ok(())
}