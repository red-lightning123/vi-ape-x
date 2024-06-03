use packets::{
    ActorConnReply, ActorSettings, CoordinatorRequest, LearnerConnReply, LearnerSettings,
    ReplayConnReply, ReplaySettings,
};
use std::net::{Ipv4Addr, TcpListener};

fn main() {
    let socket = TcpListener::bind((Ipv4Addr::UNSPECIFIED, ports::COORDINATOR)).unwrap();
    loop {
        let (stream, source_addr) = socket.accept().unwrap();
        println!("connection from {}", source_addr);
        let request = tcp_io::deserialize_from(&stream).unwrap();
        match request {
            CoordinatorRequest::ActorConn => {
                let settings = ActorSettings {
                    replay_server_addr: (Ipv4Addr::LOCALHOST, ports::REPLAY).into(),
                    learner_addr: (Ipv4Addr::LOCALHOST, ports::LEARNER).into(),
                    eps: 0.01,
                };
                let reply = ActorConnReply { settings };
                tcp_io::serialize_into(stream, &reply).unwrap();
            }
            CoordinatorRequest::LearnerConn => {
                let settings = LearnerSettings {
                    replay_server_addr: (Ipv4Addr::LOCALHOST, ports::REPLAY).into(),
                };
                let reply = LearnerConnReply { settings };
                tcp_io::serialize_into(stream, &reply).unwrap();
            }
            CoordinatorRequest::ReplayConn => {
                let settings = ReplaySettings;
                let reply = ReplayConnReply { settings };
                tcp_io::serialize_into(stream, &reply).unwrap();
            }
        }
    }
}
