use crate::{
    cmd::Args,
    run::{frontmost_app_bundle_id, run},
};
use log::{debug, error, info};
use objc2::rc::Retained;
use objc2_foundation::NSString;
use serde::{Deserialize, Serialize};
use std::{
    io::Write,
    net::{SocketAddr, TcpStream},
    sync::mpsc::{Receiver, Sender},
    time::Duration,
};

pub fn connect_kanata(args: &Args) -> TcpStream {
    info!("Connecting to kanata...");
    let kanata = TcpStream::connect_timeout(
        &SocketAddr::from(([127, 0, 0, 1], args.port)),
        Duration::from_secs(2),
    )
    .unwrap();
    info!("Connected to kanata");
    kanata
}

#[derive(Debug, Serialize, Deserialize)]
enum ClientMessage {
    ActOnFakeKey { name: String, action: Action },
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Action {
    Press,
    Release,
}

pub trait Kanata {
    fn act_on_fake_key(&mut self, name: &str, action: Action);
    fn release_vk(&mut self, name: &str);
    fn press_vk(&mut self, name: &str);
}

impl Kanata for TcpStream {
    fn act_on_fake_key(&mut self, name: &str, action: Action) {
        let msg = serde_json::to_string(&ClientMessage::ActOnFakeKey {
            name: name.to_string(),
            action,
        })
        .expect("serialize json");
        debug!("serialized json: {msg}");

        let expected_wsz = msg.len();
        let wsz = self.write(msg.as_bytes()).expect("stream writable");
        if wsz != expected_wsz {
            error!("failed to write entire message: {wsz}/{expected_wsz}")
        }

        debug!("message sent");
    }

    fn release_vk(&mut self, name: &str) {
        self.act_on_fake_key(name, Action::Release)
    }

    fn press_vk(&mut self, name: &str) {
        self.act_on_fake_key(name, Action::Press)
    }
}

trait BundleId {
    fn vk(&self, bundle_identifiers: &[String]) -> Option<String>;
}

impl BundleId for Retained<NSString> {
    fn vk(&self, bundle_identifiers: &[String]) -> Option<String> {
        let s = self.to_string();
        if bundle_identifiers.contains(&s) {
            Some(s)
        } else {
            None
        }
    }
}

pub fn kanata_appvk(args: Args, tx: Sender<Retained<NSString>>, rx: Receiver<Retained<NSString>>) {
    let mut kanata = connect_kanata(&args);
    run(tx, move || {
        let mut current_vk = frontmost_app_bundle_id().unwrap().vk(&args.bundle_ids);
        info!("initial vk: {current_vk:?}");

        for vk in &args.bundle_ids {
            kanata.release_vk(vk)
        }
        for vk in &args.bundle_ids {
            if current_vk == Some(vk.to_string()) {
                kanata.press_vk(vk)
            }
        }

        for new_bundle_id in rx {
            let new_vk = new_bundle_id.vk(&args.bundle_ids);
            if current_vk == new_vk {
                continue;
            }

            info!("{current_vk:?} -> {new_vk:?}");
            if let Some(vk) = &current_vk {
                kanata.release_vk(vk)
            }
            if let Some(vk) = &new_vk {
                kanata.press_vk(vk)
            }
            current_vk = new_vk;
        }
    });
}
