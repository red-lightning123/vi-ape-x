use model::traits::ParamFetcher;
use model::BasicModel;
use packets::{GetParamsReply, LearnerRequest};
use std::{
    net::TcpListener,
    sync::{Arc, RwLock},
    thread::JoinHandle,
};

fn spawn_batch_learner_thread(_model: Arc<RwLock<BasicModel>>) -> JoinHandle<()> {
    std::thread::spawn(move || todo!())
}

fn spawn_param_server_thread(model: Arc<RwLock<BasicModel>>) -> JoinHandle<()> {
    std::thread::spawn(move || {
        let socket = TcpListener::bind("localhost:43431").unwrap();
        loop {
            let (stream, _source_addr) = socket.accept().unwrap();
            let request = bincode::deserialize_from(&stream).unwrap();
            match request {
                LearnerRequest::GetParams => {
                    let params = {
                        let model = model.read().unwrap();
                        model.params()
                    };
                    let reply = GetParamsReply { params };
                    bincode::serialize_into(stream, &reply).unwrap();
                }
            }
        }
    })
}

fn main() {
    let model = Arc::new(RwLock::new(BasicModel::new()));
    let batch_learner_thread = spawn_batch_learner_thread(Arc::clone(&model));
    let param_server_thread = spawn_param_server_thread(model);
    batch_learner_thread.join().unwrap();
    param_server_thread.join().unwrap();
}
