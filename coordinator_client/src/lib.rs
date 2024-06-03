use packets::{ActorConnReply, CoordinatorRequest, LearnerConnReply, ReplayConnReply};
use std::net::{SocketAddr, TcpStream};

pub struct CoordinatorClient {
    server_addr: SocketAddr,
}

impl CoordinatorClient {
    pub fn new(server_addr: SocketAddr) -> Self {
        Self { server_addr }
    }
    pub fn actor_conn(&self) -> ActorConnReply {
        let request = CoordinatorRequest::ActorConn;
        let stream = match TcpStream::connect(self.server_addr) {
            Ok(stream) => stream,
            Err(e) => {
                panic!("Could not connect to coordinator: {}", e);
            }
        };
        tcp_io::serialize_into(&stream, &request).unwrap();
        tcp_io::deserialize_from(stream).unwrap()
    }
    pub fn learner_conn(&self) -> LearnerConnReply {
        let request = CoordinatorRequest::LearnerConn;
        let stream = match TcpStream::connect(self.server_addr) {
            Ok(stream) => stream,
            Err(e) => {
                panic!("Could not connect to coordinator: {}", e);
            }
        };
        tcp_io::serialize_into(&stream, &request).unwrap();
        tcp_io::deserialize_from(stream).unwrap()
    }
    pub fn replay_conn(&self) -> ReplayConnReply {
        let request = CoordinatorRequest::ReplayConn;
        let stream = match TcpStream::connect(self.server_addr) {
            Ok(stream) => stream,
            Err(e) => {
                panic!("Could not connect to coordinator: {}", e);
            }
        };
        tcp_io::serialize_into(&stream, &request).unwrap();
        tcp_io::deserialize_from(stream).unwrap()
    }
}
