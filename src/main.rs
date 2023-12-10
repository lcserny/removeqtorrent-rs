use std::sync::Arc;

use clap::Parser;
use removeqtorrent::{run, init_logging, init_config};
use tracing::info;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long)]
    hash: String
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    init_logging("removeqtorrent.log")?;
    info!("executing command");

    let args = Args::parse();
    let cfg = init_config("config/settings", "RQT")?;
    run(Arc::new(cfg), args.hash).await?;

    info!("command completed succesfully!");
    Ok(())
}