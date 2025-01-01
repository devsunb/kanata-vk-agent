use clap::Parser;
use kanata_appvk::{
    kanata::Kanata,
    util::vk,
    watch::{frontmost_app_bundle_id, input_source, watch},
};
use log::{debug, LevelFilter};
use simplelog::{ColorChoice, CombinedLogger, ConfigBuilder, TermLogger, TerminalMode};
use tokio::sync::watch::{channel, Receiver};

/// Control kanata virtual keys while observing frontmost app and input source on macOS
///
/// Example: kanata-appvk -p 5829 -b com.apple.Safari,org.mozilla.firefox -i com.apple.keylayout.ABC,com.apple.inputmethod.Korean.2SetKorean
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

    /// Input Source Identifiers, each of which is the name of a virtual key.
    #[arg(short, long, use_value_delimiter = true)]
    pub input_source_ids: Vec<String>,

    /// Just print the app's bundle id or input source id when the frontmost app and input source change. In this mode, it will not connect to kanata.
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

#[tokio::main]
async fn main() {
    let args = Args::parse();
    init_logger(&args);
    debug!("args: {:?}", args);
    // Appvk::new(args).run().await;

    let (app_tx, app_rx) = channel(frontmost_app_bundle_id().unwrap());
    let (input_source_tx, input_source_rx) = channel(input_source());

    let handle = if args.find_id_mode {
        tokio::spawn(async move { run_find_id_mode(app_rx, input_source_rx).await })
    } else {
        let mut kanata = Kanata::new(args.port);
        tokio::spawn(async move {
            run_appvk(
                args.bundle_ids,
                args.input_source_ids,
                &mut kanata,
                app_rx,
                input_source_rx,
            )
            .await
        })
    };

    let app_fn = move |bundle_id: String| {
        app_tx.send(bundle_id).unwrap();
    };
    let input_source_fn = move |input_source: String| {
        input_source_tx.send(input_source).unwrap();
    };
    watch(app_fn, input_source_fn);

    handle.await.unwrap();
}

async fn run_find_id_mode(mut app_rx: Receiver<String>, mut input_source_rx: Receiver<String>) {
    let mut current_app = frontmost_app_bundle_id().unwrap();
    println!("Frontmost App Bundle ID: {current_app}");

    let mut current_input_source = input_source();
    println!("Input Source ID: {current_input_source}");

    loop {
        tokio::select! {
            _ = app_rx.changed() => {
                let new_app = app_rx.borrow().clone();
                if current_app == new_app {
                    continue;
                }
                println!("Frontmost App Bundle ID: {new_app}");
                current_app = new_app;
            }
            _ = input_source_rx.changed() => {
                let new_input_source = input_source_rx.borrow().clone();
                if current_input_source == new_input_source {
                    continue;
                }
                println!("Input Source ID: {new_input_source}");
                current_input_source = new_input_source;
            }
        }
    }
}

async fn run_appvk(
    bundle_ids: Vec<String>,
    input_source_ids: Vec<String>,
    kanata: &mut Kanata,
    mut app_rx: Receiver<String>,
    mut input_source_rx: Receiver<String>,
) {
    let mut current_app_vk = vk(&bundle_ids, frontmost_app_bundle_id().unwrap());
    debug!("App: {current_app_vk:?}");
    kanata.init_vks(&bundle_ids, &current_app_vk);

    let mut current_input_source_vk = vk(&input_source_ids, input_source());
    debug!("Input Source: {current_input_source_vk:?}");
    kanata.init_vks(&input_source_ids, &current_input_source_vk);

    loop {
        tokio::select! {
            _ = app_rx.changed() => {
                let new_app_vk = vk(&bundle_ids, app_rx.borrow().clone());
                if current_app_vk == new_app_vk {
                    continue;
                }
                debug!("App: {current_app_vk:?} -> {new_app_vk:?}");
                if let Some(vk) = &current_app_vk {
                    kanata.release_vk(vk)
                }
                if let Some(vk) = &new_app_vk {
                    kanata.press_vk(vk)
                }
                current_app_vk = new_app_vk;
            }
            _ = input_source_rx.changed() => {
                let new_input_source_vk = vk(&input_source_ids, input_source_rx.borrow().clone());
                if current_input_source_vk == new_input_source_vk {
                    continue;
                }
                debug!("Input Source: {current_input_source_vk:?} -> {new_input_source_vk:?}");
                if let Some(vk) = &current_input_source_vk {
                    kanata.release_vk(vk)
                }
                if let Some(vk) = &new_input_source_vk {
                    kanata.press_vk(vk)
                }
                current_input_source_vk = new_input_source_vk;
            }
        }
    }
}
