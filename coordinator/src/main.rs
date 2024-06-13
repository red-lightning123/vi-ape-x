use packets::{
    ActorConnReply, ActorSettings, CoordinatorRequest, LearnerConnReply, LearnerSettings,
    PlotConnReply, PlotSettings, ReplayConnReply, ReplaySettings,
};
use std::io::Write;
use std::net::{Ipv4Addr, TcpListener};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

enum Client {
    Actor { id: usize },
    Learner,
    Replay,
    Plot,
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

fn set_term_color(stream: &mut StandardStream, color: Color) {
    stream
        .set_color(ColorSpec::new().set_bold(true).set_fg(Some(color)))
        .unwrap();
}

fn reset_term_color(stream: &mut StandardStream) {
    stream.reset().unwrap();
}

fn main() {
    let socket = TcpListener::bind((Ipv4Addr::UNSPECIFIED, ports::COORDINATOR)).unwrap();
    let mut clients = vec![];
    let mut learner_addr = None;
    let mut replay_server_addr = None;
    let mut plot_server_addr = None;
    let mut actor_id = 0;

    let mut stdout = StandardStream::stdout(ColorChoice::Always);

    loop {
        let (stream, source_addr) = socket.accept().unwrap();
        let request = tcp_io::deserialize_from(&stream).unwrap();
        match request {
            CoordinatorRequest::ActorConn => {
                set_term_color(&mut stdout, Color::Ansi256(202));
                writeln!(&mut stdout, "actor connected from {}", source_addr).unwrap();
                clients.push((stream, Client::Actor { id: actor_id }));
                actor_id += 1;
            }
            CoordinatorRequest::LearnerConn { service_addr } => {
                set_term_color(&mut stdout, Color::Ansi256(51));
                println!(
                    "learner connected from {}, serving at {}",
                    source_addr, service_addr
                );
                if learner_addr.is_some() {
                    set_term_color(&mut stdout, Color::Ansi256(210));
                    println!(
                        "rejecting learner connection from {}. another learner is already connected",
                        source_addr
                    );
                    continue;
                }
                learner_addr = Some(service_addr);
                clients.push((stream, Client::Learner));
            }
            CoordinatorRequest::ReplayConn { service_addr } => {
                set_term_color(&mut stdout, Color::Ansi256(46));
                println!(
                    "replay server connected from {}, serving at {}",
                    source_addr, service_addr
                );
                if replay_server_addr.is_some() {
                    set_term_color(&mut stdout, Color::Ansi256(210));
                    println!(
                        "rejecting replay server connection from {}. another replay server is already connected",
                        source_addr
                    );
                    continue;
                }
                replay_server_addr = Some(service_addr);
                clients.push((stream, Client::Replay));
            }
            CoordinatorRequest::PlotConn { service_addr } => {
                set_term_color(&mut stdout, Color::Ansi256(201));
                println!(
                    "plot server connected from {}, serving at {}",
                    source_addr, service_addr
                );
                if plot_server_addr.is_some() {
                    set_term_color(&mut stdout, Color::Ansi256(210));
                    println!(
                        "rejecting plot server connection from {}. another plot server is already connected",
                        source_addr
                    );
                    continue;
                }
                plot_server_addr = Some(service_addr);
                clients.push((stream, Client::Plot));
            }
            CoordinatorRequest::Start => break,
        }
    }

    let actor_count = actor_id + 1;

    for (stream, client) in clients {
        match client {
            Client::Actor { id } => {
                let settings = ActorSettings {
                    replay_server_addr,
                    learner_addr,
                    plot_server_addr,
                    id,
                    eps: compute_eps(id, actor_count),
                };
                let reply = ActorConnReply { settings };
                tcp_io::serialize_into(stream, &reply).unwrap();
            }
            Client::Learner => {
                let settings = LearnerSettings {
                    replay_server_addr,
                    plot_server_addr,
                };
                let reply = LearnerConnReply { settings };
                tcp_io::serialize_into(stream, &reply).unwrap();
            }
            Client::Replay => {
                let settings = ReplaySettings;
                let reply = ReplayConnReply {
                    settings,
                    _size_marker: u8::default(),
                };
                tcp_io::serialize_into(stream, &reply).unwrap();
            }
            Client::Plot => {
                let settings = PlotSettings { actor_count };
                let reply = PlotConnReply { settings };
                tcp_io::serialize_into(stream, &reply).unwrap();
            }
        }
    }
    reset_term_color(&mut stdout);
}
