mod cli;
mod client;
mod orchestrator;

use clap::Parser;
use cli::Cli;
use client::Config;
use client::result::DisplayOptions;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let timeout = Duration::from_secs(cli.timeout);

    eprintln!(
        "[rekor-leak] server={} api={:?} range={}..{}",
        cli.server,
        cli.api,
        cli.start,
        cli.start.saturating_add(cli.count)
    );

    let client = Config::default()
        .with_server(&cli.server)
        .with_timeout(timeout)
        .with_version(cli.api)
        .build_api()?;
    let leak_store_results = orchestrator::scan(
        client.as_ref(),
        cli.start,
        cli.count,
        usize::from(cli.concurrency),
    )
    .await;

    leak_store_results.display(&DisplayOptions {
        view_entries: cli.show_entries,
        private_only: cli.private_only,
        json: cli.json,
    });

    Ok(())
}
