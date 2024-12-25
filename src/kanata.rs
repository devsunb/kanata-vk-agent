use log::{error, info, trace};
use serde::{Deserialize, Serialize};
use std::{
    io::Write,
    net::{SocketAddr, TcpStream},
    time::Duration,
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
        let timeout = Duration::from_secs(2);

        info!("connecting to kanata: 127.0.0.1:{port}");
        let stream = TcpStream::connect_timeout(&SocketAddr::from((ip, port)), timeout)
            .expect("connect to kanata");
        info!("connected to kanata");
        Kanata { stream }
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
