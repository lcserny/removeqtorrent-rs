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

    match init_config("config/settings", "RQT") {
        Ok(cfg) => match run(cfg, args.hash) {
            Ok(_) => info!("command completed succesfully!"),
            Err(e) => log_and_fail(e, 1),
        },
        Err(e) => log_and_fail(e, 1),
    }
}


