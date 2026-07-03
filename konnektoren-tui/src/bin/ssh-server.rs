#[cfg(feature = "ssh")]
use konnektoren_tui::prelude::SshServer;

#[cfg(all(feature = "ssh", feature = "cli"))]
use clap::Parser;

#[cfg(all(feature = "ssh", feature = "cli"))]
use konnektoren_tui::prelude::Cli;

#[cfg(all(feature = "ssh", not(feature = "cli")))]
use konnektoren_tui::prelude::ManifestSourceResolver;

#[cfg(feature = "ssh")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let host = std::env::var("SSH_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = std::env::var("SSH_PORT")
        .unwrap_or_else(|_| "2222".to_string())
        .parse::<u16>()
        .unwrap_or(2222);

    tracing::info!("Starting Konnektoren SSH Server");
    tracing::info!(
        host = %if host == "0.0.0.0" { "localhost" } else { &host },
        port,
        "Server listening — connect with: ssh -p {port} <username>@{}",
        if host == "0.0.0.0" { "localhost" } else { &host }
    );
    tracing::info!("Press Ctrl+C to stop the server");

    let manifest_source = {
        #[cfg(feature = "cli")]
        {
            Cli::parse().manifest_source()
        }

        #[cfg(not(feature = "cli"))]
        {
            ManifestSourceResolver::default().resolve_from_env()?
        }
    };

    if let Some(manifest_source) = manifest_source {
        SshServer::run_with_manifest_source(&host, port, manifest_source).await?;
    } else {
        SshServer::run(&host, port).await?;
    }

    Ok(())
}

#[cfg(not(feature = "ssh"))]
fn main() {
    eprintln!("This binary requires the 'ssh' feature to be enabled.");
    eprintln!("Build with: cargo build --bin ssh-server --features ssh");
    std::process::exit(1);
}
