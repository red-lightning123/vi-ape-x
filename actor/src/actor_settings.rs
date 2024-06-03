use std::net::SocketAddr;

pub struct ActorSettings {
    pub learner_addr: SocketAddr,
    pub replay_server_addr: SocketAddr,
    pub eps: f64,
}
