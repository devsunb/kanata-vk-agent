use clap::Parser;
use kanata_appvk::{
    cmd::Args,
    kanata::Kanata,
    log::init_logger,
    watch::{frontmost_app_bundle_id, watch},
};
use std::{sync::mpsc, thread};

fn main() {
    let args = Args::parse();
    init_logger(&args);
    log::debug!("args: {:?}", args);
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
        let (tx, rx) = mpsc::channel();

        let handle = if self.find_id_mode {
            thread::spawn(move || Appvk::run_find_id_mode(rx))
        } else {
            let kanata = Kanata::connect(self.port);
            thread::spawn(move || self.run_appvk(kanata, rx))
        };

        watch(tx);
        handle.join().unwrap();
    }

    fn run_find_id_mode(rx: mpsc::Receiver<String>) {
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

    fn run_appvk(&self, mut kanata: Kanata, rx: mpsc::Receiver<String>) {
        let mut current_vk = self.to_vk(frontmost_app_bundle_id().unwrap());
        log::debug!("initial vk: {current_vk:?}");

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

            log::debug!("{current_vk:?} -> {new_vk:?}");
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
