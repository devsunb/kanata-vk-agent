use clap::Parser;
use kanata_appvk::{cmd::Args, kanata::kanata_appvk, log::init_logger, run::id_mode};
use log::debug;
use std::sync::mpsc;

fn main() {
    let args = Args::parse();
    init_logger(&args);
    debug!("args: {:?}", args);

    let (tx, rx) = mpsc::channel();
    match args.find_id_mode {
        true => id_mode(tx, rx),
        false => kanata_appvk(args, tx, rx),
    }
}
