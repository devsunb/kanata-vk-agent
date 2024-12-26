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
    stream: TcpStream,
}

impl Kanata {
    pub fn connect(port: u16) -> Self {
        let ip = [127, 0, 0, 1];
        let timeout = Duration::from_millis(200);
        let retry_interval = Duration::from_millis(200);
        let retry_timeout = Duration::from_secs(2);

        info!("connecting to kanata: 127.0.0.1:{port}");
        let start_time = Instant::now();
        loop {
            if start_time.elapsed() >= retry_timeout {
                panic!("failed to connect to kanata within 2 seconds");
            }
            match TcpStream::connect_timeout(&SocketAddr::from((ip, port)), timeout) {
                Ok(stream) => {
                    info!("connected to kanata");
                    return Kanata { stream };
                }
                Err(e) => {
                    debug!("failed to connect to kanata: {e}");
                    thread::sleep(retry_interval);
                }
            }
        }
    }

    fn act_on_fake_key(&mut self, name: &str, action: Action) {
        let msg = serde_json::to_string(&ClientMessage::ActOnFakeKey {
            name: name.to_string(),
            action,
        })
        .expect("serialize json");
        trace!("serialized json: {msg}");

        let expected_wsz = msg.len();
        let wsz = self
            .stream
            .write(msg.as_bytes())
            .expect("write message to kanata");
        if wsz != expected_wsz {
            error!("failed to write entire message: {wsz}/{expected_wsz}")
        }
    }

    pub fn release_vk(&mut self, name: &str) {
        self.act_on_fake_key(name, Action::Release)
    }

    pub fn press_vk(&mut self, name: &str) {
        self.act_on_fake_key(name, Action::Press)
    }
}
