use packets::{
    ActorConnReply, ActorSettings, CoordinatorRequest, LearnerConnReply, LearnerSettings,
    ReplayConnReply, ReplaySettings,
};
use std::net::{Ipv4Addr, TcpListener};

enum Client {
    Actor,
    Learner,
    Replay,
}

fn main() {
    let socket = TcpListener::bind((Ipv4Addr::UNSPECIFIED, ports::COORDINATOR)).unwrap();
    let mut clients = vec![];
    let mut learner_addr = None;
    let mut replay_server_addr = None;

    loop {
        let (stream, source_addr) = socket.accept().unwrap();
        println!("connection from {}", source_addr);
        let request = tcp_io::deserialize_from(&stream).unwrap();
        match request {
            CoordinatorRequest::ActorConn => {
                clients.push((stream, Client::Actor));
            }
            CoordinatorRequest::LearnerConn => {
                if learner_addr.is_some() {
                    continue;
                }
                learner_addr = Some((source_addr.ip(), ports::LEARNER).into());
                clients.push((stream, Client::Learner));
            }
            CoordinatorRequest::ReplayConn => {
                if replay_server_addr.is_some() {
                    continue;
                }
                replay_server_addr = Some((source_addr.ip(), ports::REPLAY).into());
                clients.push((stream, Client::Replay));
            }
            CoordinatorRequest::Start => break,
        }
    }

    let learner_addr = match learner_addr {
        Some(addr) => addr,
        None => {
            println!("attempted to start but no learner connected. aborting...");
            return;
        }
    };

    let replay_server_addr = match replay_server_addr {
        Some(addr) => addr,
        None => {
            println!("attempted to start but no replay server connected. aborting...");
            return;
        }
    };

    for (stream, client) in clients {
        match client {
            Client::Actor => {
                let settings = ActorSettings {
                    replay_server_addr,
                    learner_addr,
                    eps: 0.01,
                };
                let reply = ActorConnReply { settings };
                tcp_io::serialize_into(stream, &reply).unwrap();
            }
            Client::Learner => {
                let settings = LearnerSettings { replay_server_addr };
                let reply = LearnerConnReply { settings };
                tcp_io::serialize_into(stream, &reply).unwrap();
            }
            Client::Replay => {
                let settings = ReplaySettings;
                let reply = ReplayConnReply { settings };
                tcp_io::serialize_into(stream, &reply).unwrap();
            }
        }
    }
}
