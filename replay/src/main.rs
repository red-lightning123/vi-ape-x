use packets::ReplayRequest;
use std::net::TcpListener;

fn main() {
    let socket = TcpListener::bind("localhost:43430").unwrap();
    loop {
        let (stream, _source_addr) = socket.accept().unwrap();
        let request = bincode::deserialize_from(&stream).unwrap();
        match request {
            ReplayRequest::ReleaseLock => todo!(),
            ReplayRequest::SampleBatch { batch_len: _ } => todo!(),
            ReplayRequest::InsertBatch { batch: _ } => todo!(),
            ReplayRequest::UpdateBatchPriorities { batch: _ } => todo!(),
        }
    }
}
