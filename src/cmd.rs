use clap::Parser;

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
