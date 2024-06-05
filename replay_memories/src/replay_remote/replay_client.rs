use packets::{Insertion, PriorityUpdate, ReplayRequest, SampleBatchResult};
use std::net::{SocketAddr, TcpStream};

pub struct ReplayClient {
    server_addr: SocketAddr,
}

impl ReplayClient {
    pub fn new(server_addr: SocketAddr) -> Self {
        Self { server_addr }
    }
    pub fn truncate(&mut self) {
        let request = ReplayRequest::Truncate;
        let stream = match TcpStream::connect(self.server_addr) {
            Ok(stream) => stream,
            Err(e) => {
                panic!("Could not connect to replay server: {}", e);
            }
        };
        tcp_io::serialize_into(stream, &request).unwrap();
    }
    pub fn update_priorities(&mut self, batch: Vec<PriorityUpdate>) {
        let request = ReplayRequest::UpdateBatchPriorities { batch };
        let stream = match TcpStream::connect(self.server_addr) {
            Ok(stream) => stream,
            Err(e) => {
                panic!("Could not connect to replay server: {}", e);
            }
        };
        tcp_io::serialize_into(stream, &request).unwrap();
    }
    pub fn insert(&mut self, batch: Vec<Insertion>) {
        let request = ReplayRequest::InsertBatch { batch };
        let stream = match TcpStream::connect(self.server_addr) {
            Ok(stream) => stream,
            Err(e) => {
                panic!("Could not connect to replay server: {}", e);
            }
        };
        tcp_io::serialize_into(stream, &request).unwrap();
    }
    pub fn sample_batch(&self, batch_len: usize) -> SampleBatchResult {
        let request = ReplayRequest::SampleBatch { batch_len };
        let stream = match TcpStream::connect(self.server_addr) {
            Ok(stream) => stream,
            Err(e) => {
                panic!("Could not connect to replay server: {}", e);
            }
        };
        tcp_io::serialize_into(&stream, &request).unwrap();
        tcp_io::deserialize_from(stream).unwrap()
    }
}
