use clap::Parser;
use kanata_appvk::{
    kanata::Kanata,
    watch::{frontmost_app_bundle_id, watch},
};
use log::{debug, LevelFilter};
use simplelog::{ColorChoice, CombinedLogger, ConfigBuilder, TermLogger, TerminalMode};
use std::{
    sync::mpsc::{channel, Receiver},
    thread,
};

/// Watch macOS frontmost app and press/release kanata virtual keys
///
/// Example: kanata-appvk -p 5829 -b com.apple.Safari,org.mozilla.firefox
#[derive(Parser, Debug)]
#[command(version, about)]
pub struct Args {
    /// Log level
    #[arg(short, long, value_enum, default_value_t)]
    pub log_level: LogLevel,

    /// TCP port number of kanata
    #[arg(short, long, value_parser = clap::value_parser!(u16).range(1..), default_value_t = 5829)]
    pub port: u16,

    /// Bundle Identifiers, each of which is the name of a virtual key.
    #[arg(short, long, use_value_delimiter = true)]
    pub bundle_ids: Vec<String>,

    /// Just print frontmost app's Bundle Identifier when it changes without connecting to kanata.
    #[arg(short, long)]
    pub find_id_mode: bool,
}

#[derive(clap::ValueEnum, Default, Clone, Debug)]
pub enum LogLevel {
    Off,
    Error,
    Warn,
    #[default]
    Info,
    Debug,
    Trace,
}

pub fn init_logger(args: &Args) {
    let mut config = ConfigBuilder::new();
    if let Err(e) = config.set_time_offset_to_local() {
        eprintln!("WARNING: failed to set logger timezone to local: {e:?}");
    };
    CombinedLogger::init(vec![TermLogger::new(
        match args.log_level {
            LogLevel::Off => LevelFilter::Off,
            LogLevel::Error => LevelFilter::Error,
            LogLevel::Warn => LevelFilter::Warn,
            LogLevel::Info => LevelFilter::Info,
            LogLevel::Debug => LevelFilter::Debug,
            LogLevel::Trace => LevelFilter::Trace,
        },
        config.build(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )])
    .expect("init logger");
}

fn main() {
    let args = Args::parse();
    init_logger(&args);
    debug!("args: {:?}", args);
    Appvk::new(args).run();
}

pub struct Appvk {
    find_id_mode: bool,
    bundle_ids: Vec<String>,
    port: u16,
}

impl Appvk {
    pub fn new(args: Args) -> Self {
        Self {
            find_id_mode: args.find_id_mode,
            bundle_ids: args.bundle_ids,
            port: args.port,
        }
    }

    fn to_vk(&self, bundle_id: String) -> Option<String> {
        if self.bundle_ids.contains(&bundle_id) {
            Some(bundle_id)
        } else {
            None
        }
    }

    pub fn run(self) {
        let (tx, rx) = channel();

        let handle = if self.find_id_mode {
            thread::spawn(move || Appvk::run_find_id_mode(rx))
        } else {
            let kanata = Kanata::connect(self.port);
            thread::spawn(move || self.run_appvk(kanata, rx))
        };

        watch(tx);
        handle.join().unwrap();
    }

    fn run_find_id_mode(rx: Receiver<String>) {
        let mut current = frontmost_app_bundle_id().unwrap();
        println!("{current}");
        for new in rx {
            let new = new.to_string();
            if current == new {
                continue;
            }
            println!("{new}");
            current = new;
        }
    }

    fn run_appvk(&self, mut kanata: Kanata, rx: Receiver<String>) {
        let mut current_vk = self.to_vk(frontmost_app_bundle_id().unwrap());
        debug!("initial vk: {current_vk:?}");

        for vk in &self.bundle_ids {
            kanata.release_vk(vk)
        }
        for vk in &self.bundle_ids {
            if current_vk == Some(vk.to_string()) {
                kanata.press_vk(vk)
            }
        }

        for new in rx {
            let new_vk = self.to_vk(new);
            if current_vk == new_vk {
                continue;
            }

            debug!("{current_vk:?} -> {new_vk:?}");
            if let Some(vk) = &current_vk {
                kanata.release_vk(vk)
            }
            if let Some(vk) = &new_vk {
                kanata.press_vk(vk)
            }
            current_vk = new_vk;
        }
    }
}
