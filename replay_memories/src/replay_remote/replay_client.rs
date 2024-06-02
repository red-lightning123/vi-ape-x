use packets::{Insertion, PriorityUpdate, ReplayRequest, SampleBatchResult};
use std::net::TcpStream;

pub struct ReplayClient {}

impl ReplayClient {
    pub fn new() -> Self {
        Self {}
    }
    pub fn update_priorities(&mut self, batch: Vec<PriorityUpdate>) {
        let request = ReplayRequest::UpdateBatchPriorities { batch };
        let stream = match TcpStream::connect("localhost:43430") {
            Ok(stream) => stream,
            Err(e) => {
                panic!("Could not connect to replay server: {}", e);
            }
        };
        tcp_io::serialize_into(stream, &request).unwrap();
    }
    pub fn insert(&mut self, batch: Vec<Insertion>) {
        let request = ReplayRequest::InsertBatch { batch };
        let stream = match TcpStream::connect("localhost:43430") {
            Ok(stream) => stream,
            Err(e) => {
                panic!("Could not connect to replay server: {}", e);
            }
        };
        tcp_io::serialize_into(stream, &request).unwrap();
    }
    pub fn sample_batch(&self, batch_len: usize) -> SampleBatchResult {
        let request = ReplayRequest::SampleBatch { batch_len };
        let stream = match TcpStream::connect("localhost:43430") {
            Ok(stream) => stream,
            Err(e) => {
                panic!("Could not connect to replay server: {}", e);
            }
        };
        tcp_io::serialize_into(&stream, &request).unwrap();
        tcp_io::deserialize_from(stream).unwrap()
    }
}
