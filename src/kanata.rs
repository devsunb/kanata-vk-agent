use log::{debug, error, info, trace};
use serde::{Deserialize, Serialize};
use std::{
    io::Write,
    net::{SocketAddr, TcpStream},
    thread,
    time::{Duration, Instant},
};

#[derive(Debug, Serialize, Deserialize)]
enum ClientMessage {
    ActOnFakeKey { name: String, action: Action },
}

#[derive(Debug, Serialize, Deserialize)]
enum Action {
    Press,
    Release,
}

pub struct Kanata {
    addr: SocketAddr,
    stream: TcpStream,
}

impl Kanata {
    pub fn new(port: u16) -> Self {
        let addr = SocketAddr::from(([127, 0, 0, 1], port));
        let stream = Self::connect(&addr);
        Kanata { addr, stream }
    }

    pub fn connect(addr: &SocketAddr) -> TcpStream {
        let timeout = Duration::from_millis(200);
        let retry_interval = Duration::from_millis(200);
        let retry_timeout = Duration::from_secs(2);

        info!("connecting to kanata: {addr}");
        let start_time = Instant::now();
        loop {
            if start_time.elapsed() >= retry_timeout {
                panic!("failed to connect to kanata within 2 seconds");
            }
            match TcpStream::connect_timeout(addr, timeout) {
                Ok(stream) => {
                    info!("connected to kanata");
                    return stream;
                }
                Err(e) => {
                    debug!("failed to connect to kanata: {e}");
                    thread::sleep(retry_interval);
                }
            }
        }
    }

    fn reconnect(&mut self) {
        self.stream = Self::connect(&self.addr);
    }

    fn act_on_fake_key(&mut self, name: &str, action: Action) {
        let msg = serde_json::to_string(&ClientMessage::ActOnFakeKey {
            name: name.to_string(),
            action,
        })
        .expect("serialize json");
        trace!("serialized json: {msg}");

        let expected_wsz = msg.len();
        match self.stream.write(msg.as_bytes()) {
            Ok(wsz) => {
                trace!("wrote message: {wsz}");
                if wsz != expected_wsz {
                    error!("failed to write entire message: {wsz}/{expected_wsz}");
                }
            }
            Err(e) => {
                error!("failed to write message to kanata: {e}");
                self.reconnect();
            }
        }
    }

    pub fn release_vk(&mut self, name: &str) {
        self.act_on_fake_key(name, Action::Release)
    }

    pub fn press_vk(&mut self, name: &str) {
        self.act_on_fake_key(name, Action::Press)
    }

    pub fn init_vks(&mut self, vks: &Vec<String>, current_vk: &Option<String>) {
        if let Some(current_vk) = current_vk {
            for vk in vks {
                if vk == current_vk {
                    self.press_vk(vk)
                } else {
                    self.release_vk(vk)
                }
            }
        }
    }
}
