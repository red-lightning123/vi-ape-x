mod learner_schedule;

use learner_schedule::LearnerSchedule;
use model::traits::{ParamFetcher, TargetNet};
use model::BasicModel;
use packets::{GetParamsReply, LearnerRequest};
use prompt::prompt_user_for_service_ip_addr;
use replay_wrappers::RemoteReplayWrapper;
use std::net::{Ipv4Addr, SocketAddr, TcpListener};
use std::sync::{Arc, RwLock};
use std::thread::JoinHandle;

fn spawn_batch_learner_thread(
    agent: Arc<RwLock<RemoteReplayWrapper<BasicModel>>>,
) -> JoinHandle<()> {
    std::thread::spawn(move || {
        const TARGET_UPDATE_INTERVAL_STEPS: u32 = 2_500;
        const BETA: f64 = 0.4;
        let mut schedule = LearnerSchedule::new(TARGET_UPDATE_INTERVAL_STEPS);
        loop {
            {
                let mut agent = agent.write().unwrap();
                agent.train_step(BETA);
                if schedule.is_time_to_update_target() {
                    agent.copy_control_to_target();
                }
            }
            schedule.step();
        }
    })
}

fn spawn_param_server_thread(
    agent: Arc<RwLock<RemoteReplayWrapper<BasicModel>>>,
) -> JoinHandle<()> {
    std::thread::spawn(move || {
        let socket = TcpListener::bind((Ipv4Addr::UNSPECIFIED, ports::LEARNER)).unwrap();
        loop {
            let (stream, _source_addr) = socket.accept().unwrap();
            let request = tcp_io::deserialize_from(&stream).unwrap();
            match request {
                LearnerRequest::GetParams => {
                    let params = {
                        let agent = agent.read().unwrap();
                        agent.params()
                    };
                    let reply = GetParamsReply { params };
                    tcp_io::serialize_into(stream, &reply).unwrap();
                }
            }
        }
    })
}

fn main() {
    let replay_server_ip_addr = prompt_user_for_service_ip_addr("replay server");
    println!("replay server ip addr set to {}...", replay_server_ip_addr);
    let replay_server_addr = (replay_server_ip_addr, ports::REPLAY).into();
    run(replay_server_addr);
}

fn run(replay_server_addr: SocketAddr) {
    const ALPHA: f64 = 0.6;
    let agent = Arc::new(RwLock::new(RemoteReplayWrapper::wrap(
        BasicModel::new(),
        replay_server_addr,
        ALPHA,
    )));
    let batch_learner_thread = spawn_batch_learner_thread(Arc::clone(&agent));
    let param_server_thread = spawn_param_server_thread(agent);
    batch_learner_thread.join().unwrap();
    param_server_thread.join().unwrap();
}
