#[cfg(feature = "crossterm")]
use konnektoren_tui::prelude::{App, init, restore};

#[cfg(all(feature = "crossterm", feature = "cli"))]
use clap::Parser;

#[cfg(all(feature = "crossterm", feature = "cli"))]
use konnektoren_tui::prelude::Cli;

#[cfg(all(feature = "crossterm", not(feature = "cli")))]
use konnektoren_tui::prelude::load_session_from_env;

#[cfg(feature = "crossterm")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let session = {
        #[cfg(feature = "cli")]
        {
            Cli::parse().load_session()?
        }

        #[cfg(not(feature = "cli"))]
        {
            load_session_from_env()?
        }
    };

    let mut app = if let Some(session) = session {
        App::with_session(session)
    } else {
        App::new()
    };
    let mut terminal = init()?;
    app.run(&mut terminal)?;
    restore()?;
    Ok(())
}

#[cfg(not(feature = "crossterm"))]
fn main() {
    eprintln!("This binary requires the 'crossterm' feature to be enabled.");
    eprintln!("Run with: cargo run --features crossterm");
    std::process::exit(1);
}
