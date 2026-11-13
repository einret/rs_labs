use clap::Parser;

use crate::client::{RekorVersion, http::DEFAULT_SERVER};

#[derive(Debug, Parser)]
#[command(
    name = "rekor-leak",
    bin_name = "rekor-leak",
    version,
    about = "Scan public Rekor logs for leaked CI/CD repository metadata",
    after_help = "Read-only. Respect the target's rate limits. Defaults to a small scan."
)]
pub struct Cli {
    #[arg(short, long, default_value = DEFAULT_SERVER)]
    pub server: String,

    #[arg(long, value_enum, default_value_t = RekorVersion::V2)]
    pub api: RekorVersion,

    #[arg(long, default_value_t = 0)]
    pub start: u64,

    #[arg(
        short = 'n',
        long,
        default_value_t = 1_000,
        value_parser = clap::value_parser!(u64).range(1..)
    )]
    pub count: u64,

    #[arg(
        short = 'j',
        long,
        default_value_t = 8,
        value_parser = clap::value_parser!(u16).range(1..=8)
    )]
    pub concurrency: u16,

    #[arg(long, default_value_t = 10)]
    pub timeout: u64,

    #[arg(long)]
    pub json: bool,

    #[arg(long)]
    pub show_entries: bool,

    #[arg(long)]
    pub private_only: bool,
}
