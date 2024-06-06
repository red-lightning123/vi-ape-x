use packets::{
    ActorConnReply, ActorSettings, CoordinatorRequest, LearnerConnReply, LearnerSettings,
    PlotConnReply, PlotSettings, ReplayConnReply, ReplaySettings,
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
    pub fn learner_conn(&self, service_addr: SocketAddr) -> LearnerSettings {
        let request = CoordinatorRequest::LearnerConn { service_addr };
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
    pub fn replay_conn(&self, service_addr: SocketAddr) -> ReplaySettings {
        let request = CoordinatorRequest::ReplayConn { service_addr };
        let stream = match TcpStream::connect(self.server_addr) {
            Ok(stream) => stream,
            Err(e) => {
                panic!("Could not connect to coordinator: {}", e);
            }
        };
        tcp_io::serialize_into(&stream, &request).unwrap();
        let ReplayConnReply {
            settings,
            _size_marker,
        } = tcp_io::deserialize_from(stream).unwrap();
        settings
    }
    pub fn plot_conn(&self, service_addr: SocketAddr) -> PlotSettings {
        let request = CoordinatorRequest::PlotConn { service_addr };
        let stream = match TcpStream::connect(self.server_addr) {
            Ok(stream) => stream,
            Err(e) => {
                panic!("Could not connect to coordinator: {}", e);
            }
        };
        tcp_io::serialize_into(&stream, &request).unwrap();
        let PlotConnReply {
            settings,
            _size_marker,
        } = tcp_io::deserialize_from(stream).unwrap();
        settings
    }
    pub fn start(&self) {
        let request = CoordinatorRequest::Start;
        let stream = match TcpStream::connect(self.server_addr) {
            Ok(stream) => stream,
            Err(e) => {
                panic!("Could not connect to coordinator: {}", e);
            }
        };
        tcp_io::serialize_into(&stream, &request).unwrap();
    }
}
