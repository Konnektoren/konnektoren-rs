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
            let cli = Cli::parse();
            let language = cli.language().map(str::to_string);
            (cli.load_session()?, language)
        }

        #[cfg(not(feature = "cli"))]
        {
            (load_session_from_env()?, None)
        }
    };

    let mut app = if let Some(session) = session.0 {
        App::with_session(session)
    } else {
        App::new()
    };
    if let Some(language) = session.1 {
        app.set_language(language);
    }
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
