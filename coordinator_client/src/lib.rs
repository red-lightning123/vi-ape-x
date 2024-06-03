use packets::{
    ActorConnReply, ActorSettings, CoordinatorRequest, LearnerConnReply, LearnerSettings,
    ReplayConnReply, ReplaySettings,
};
use std::net::{SocketAddr, TcpStream};

pub struct CoordinatorClient {
    server_addr: SocketAddr,
}

impl CoordinatorClient {
    pub fn new(server_addr: SocketAddr) -> Self {
        Self { server_addr }
    }
    pub fn actor_conn(&self) -> ActorSettings {
        let request = CoordinatorRequest::ActorConn;
        let stream = match TcpStream::connect(self.server_addr) {
            Ok(stream) => stream,
            Err(e) => {
                panic!("Could not connect to coordinator: {}", e);
            }
        };
        tcp_io::serialize_into(&stream, &request).unwrap();
        let ActorConnReply { settings } = tcp_io::deserialize_from(stream).unwrap();
        settings
    }
    pub fn learner_conn(&self, service_port: u16) -> LearnerSettings {
        let request = CoordinatorRequest::LearnerConn { service_port };
        let stream = match TcpStream::connect(self.server_addr) {
            Ok(stream) => stream,
            Err(e) => {
                panic!("Could not connect to coordinator: {}", e);
            }
        };
        tcp_io::serialize_into(&stream, &request).unwrap();
        let LearnerConnReply { settings } = tcp_io::deserialize_from(stream).unwrap();
        settings
    }
    pub fn replay_conn(&self, service_port: u16) -> ReplaySettings {
        let request = CoordinatorRequest::ReplayConn { service_port };
        let stream = match TcpStream::connect(self.server_addr) {
            Ok(stream) => stream,
            Err(e) => {
                panic!("Could not connect to coordinator: {}", e);
            }
        };
        tcp_io::serialize_into(&stream, &request).unwrap();
        let ReplayConnReply { settings } = tcp_io::deserialize_from(stream).unwrap();
        settings
    }
}
