use clap::Parser;
use schemagen::driver::{self, Args};

mod config;
mod policy;

use policy::TairuPolicy;

/// Generates `src/generated.rs` from the 3D Tiles schemas.
///
/// The config lives in `config.rs`; a `--config` file is ignored.
#[derive(Parser)]
#[command(name = "xtask")]
struct Cli {
    #[command(flatten)]
    args: Args,
}

fn main() {
    let cli = Cli::parse();
    let result =
        config::config().and_then(|config| driver::run_with(&cli.args, &config, &TairuPolicy));
    if let Err(error) = result {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}
