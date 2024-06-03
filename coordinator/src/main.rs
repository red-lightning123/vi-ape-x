use packets::{
    ActorConnReply, ActorSettings, CoordinatorRequest, LearnerConnReply, LearnerSettings,
    ReplayConnReply, ReplaySettings,
};
use std::net::{Ipv4Addr, TcpListener};

enum Client {
    Actor { id: usize },
    Learner,
    Replay,
}

// Eps is computed according to the Ape-X paper
fn compute_eps(actor_id: usize, actor_count: usize) -> f64 {
    const EPS_BASE: f64 = 0.4;
    const ALPHA: f64 = 7.0;
    match actor_count {
        0 => unreachable!(),
        // The actual formula is undefined for a single actor. We arbitrarily
        // set it to EPS_BASE
        1 => EPS_BASE,
        _ => EPS_BASE.powf(1.0 + (actor_id as f64 * ALPHA) / (actor_count as f64 - 1.0)),
    }
}

fn main() {
    let socket = TcpListener::bind((Ipv4Addr::UNSPECIFIED, ports::COORDINATOR)).unwrap();
    let mut clients = vec![];
    let mut learner_addr = None;
    let mut replay_server_addr = None;
    let mut actor_id = 0;

    loop {
        let (stream, source_addr) = socket.accept().unwrap();
        let request = tcp_io::deserialize_from(&stream).unwrap();
        match request {
            CoordinatorRequest::ActorConn => {
                println!("actor connected from {}", source_addr);
                clients.push((stream, Client::Actor { id: actor_id }));
                actor_id += 1;
            }
            CoordinatorRequest::LearnerConn { service_port } => {
                println!("learner connected from {}", source_addr);
                if learner_addr.is_some() {
                    println!(
                        "rejecting learner at {}. another learner is already connected",
                        source_addr
                    );
                    continue;
                }
                learner_addr = Some((source_addr.ip(), service_port).into());
                clients.push((stream, Client::Learner));
            }
            CoordinatorRequest::ReplayConn { service_port } => {
                println!("replay server connected from {}", source_addr);
                if replay_server_addr.is_some() {
                    println!(
                        "rejecting replay server at {}. another replay server is already connected",
                        source_addr
                    );
                    continue;
                }
                replay_server_addr = Some((source_addr.ip(), service_port).into());
                clients.push((stream, Client::Replay));
            }
            CoordinatorRequest::Start => break,
        }
    }

    let actor_count = actor_id + 1;

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
            Client::Actor { id } => {
                let settings = ActorSettings {
                    replay_server_addr,
                    learner_addr,
                    eps: compute_eps(id, actor_count),
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
