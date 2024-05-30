use model::traits::ParamFetcher;
use model::BasicModel;
use packets::{GetParamsReply, LearnerRequest};
use std::net::TcpListener;

fn main() {
    let socket = TcpListener::bind("localhost:43431").unwrap();
    let model = BasicModel::new();
    loop {
        let (stream, _source_addr) = socket.accept().unwrap();
        let request = bincode::deserialize_from(&stream).unwrap();
        match request {
            LearnerRequest::GetParams => {
                let reply = GetParamsReply {
                    params: model.params(),
                };
                bincode::serialize_into(stream, &reply).unwrap();
            }
        }
    }
}
