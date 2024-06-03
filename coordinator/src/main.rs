use packets::{ActorConnReply, CoordinatorRequest, LearnerConnReply, ReplayConnReply};
use std::net::{Ipv4Addr, TcpListener};

fn main() {
    let socket = TcpListener::bind((Ipv4Addr::UNSPECIFIED, ports::COORDINATOR)).unwrap();
    loop {
        let (stream, source_addr) = socket.accept().unwrap();
        println!("connection from {}", source_addr);
        let request = tcp_io::deserialize_from(&stream).unwrap();
        match request {
            CoordinatorRequest::ActorConn => {
                let reply = ActorConnReply {
                    replay_server_addr: (Ipv4Addr::LOCALHOST, ports::REPLAY).into(),
                    learner_addr: (Ipv4Addr::LOCALHOST, ports::LEARNER).into(),
                    eps: 0.01,
                };
                tcp_io::serialize_into(stream, &reply).unwrap();
            }
            CoordinatorRequest::LearnerConn => {
                let reply = LearnerConnReply {
                    replay_server_addr: (Ipv4Addr::LOCALHOST, ports::REPLAY).into(),
                };
                tcp_io::serialize_into(stream, &reply).unwrap();
            }
            CoordinatorRequest::ReplayConn => {
                let reply = ReplayConnReply;
                tcp_io::serialize_into(stream, &reply).unwrap();
            }
        }
    }
}
