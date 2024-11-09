use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Serialize, Deserialize, Debug)]
pub struct P2PManager {
    peers: Vec<Peer>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Peer {
    endpoints: Vec<Url>,
    current_endpoint: usize,
    // TODO: Implement Wireguard tunnels
}

impl Peer {
    pub fn get_current_endpoint(&self) -> Url {
        self.endpoints[self.current_endpoint].clone()
    }
    pub fn connect(&mut self) {
        todo!()
    }
    pub fn disconnect(&mut self) {
        todo!()
    }
}
