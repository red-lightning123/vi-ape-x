use model::traits::ParamFetcher;
use model::BasicModel;
use packets::{GetParamsReply, LearnerRequest};
use replay_wrappers::RemoteReplayWrapper;
use std::{
    net::TcpListener,
    sync::{Arc, RwLock},
    thread::JoinHandle,
};

fn spawn_batch_learner_thread(
    agent: Arc<RwLock<RemoteReplayWrapper<BasicModel>>>,
) -> JoinHandle<()> {
    std::thread::spawn(move || todo!())
}

fn spawn_param_server_thread(
    agent: Arc<RwLock<RemoteReplayWrapper<BasicModel>>>,
) -> JoinHandle<()> {
    std::thread::spawn(move || {
        let socket = TcpListener::bind("localhost:43431").unwrap();
        loop {
            let (stream, _source_addr) = socket.accept().unwrap();
            let request = bincode::deserialize_from(&stream).unwrap();
            match request {
                LearnerRequest::GetParams => {
                    let params = {
                        let agent = agent.read().unwrap();
                        agent.params()
                    };
                    let reply = GetParamsReply { params };
                    bincode::serialize_into(stream, &reply).unwrap();
                }
            }
        }
    })
}

fn main() {
    const ALPHA: f64 = 0.6;
    let agent = Arc::new(RwLock::new(RemoteReplayWrapper::wrap(
        BasicModel::new(),
        ALPHA,
    )));
    let batch_learner_thread = spawn_batch_learner_thread(Arc::clone(&agent));
    let param_server_thread = spawn_param_server_thread(agent);
    batch_learner_thread.join().unwrap();
    param_server_thread.join().unwrap();
}
