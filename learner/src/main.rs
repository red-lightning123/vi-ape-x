use model::BasicModel;
use packets::LearnerRequest;
use std::net::TcpListener;

fn main() {
    let socket = TcpListener::bind("localhost:43431").unwrap();
    let _model = BasicModel::new();
    loop {
        let (stream, _source_addr) = socket.accept().unwrap();
        let request = bincode::deserialize_from(stream).unwrap();
        match request {
            LearnerRequest::GetParams => todo!(),
        }
    }
}
