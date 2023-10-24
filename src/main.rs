use clap::Parser;
use removeqtorrent::{run, log_and_fail, init_logging, init_config};
use tracing::info;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long)]
    hash: String
}

fn main() {
    init_logging(".", "removeqtorrent.log");
    info!("executing command");

    let args = Args::parse();
    let cfg = init_config("config/settings", "RQT")
        .map_err(|e| log_and_fail(e, 1)).unwrap();
    run(cfg, args.hash).map_err(|e| log_and_fail(e, 1)).unwrap();

    info!("command completed succesfully!");
}