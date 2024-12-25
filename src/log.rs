use crate::cmd::{Args, LogLevel};
use simplelog::{ColorChoice, CombinedLogger, ConfigBuilder, TermLogger, TerminalMode};

pub fn init_logger(args: &Args) {
    let mut config = ConfigBuilder::new();
    if let Err(e) = config.set_time_offset_to_local() {
        eprintln!("WARNING: could not set log TZ to local: {:?}", e);
    };
    CombinedLogger::init(vec![TermLogger::new(
        match args.log_level {
            LogLevel::Off => log::LevelFilter::Off,
            LogLevel::Error => log::LevelFilter::Error,
            LogLevel::Warn => log::LevelFilter::Warn,
            LogLevel::Info => log::LevelFilter::Info,
            LogLevel::Debug => log::LevelFilter::Debug,
            LogLevel::Trace => log::LevelFilter::Trace,
        },
        config.build(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )])
    .expect("init logger");
}
