use packets::{Insertion, ReplayRequest, SampleBatchResult};
use replay_data::CompressedTransition;
use std::{net::TcpStream, path::Path};

pub struct ReplayRemote {}

impl ReplayRemote {
    pub fn new() -> Self {
        Self {}
    }
    pub fn update_priorities_with_td_errors(
        &mut self,
        _indices: &[usize],
        _abs_td_errors: &[f64],
        _alpha: f64,
    ) {
        todo!()
    }
    pub fn add_transition_with_priority(
        &mut self,
        transition: CompressedTransition,
        priority: f64,
    ) {
        let request = ReplayRequest::InsertBatch {
            batch: vec![Insertion {
                priority,
                transition,
            }],
        };
        let stream = match TcpStream::connect("localhost:43430") {
            Ok(stream) => stream,
            Err(e) => {
                panic!("Could not connect to replay server: {}", e);
            }
        };
        bincode::serialize_into(stream, &request).unwrap();
    }
    pub fn sample_batch(&self, batch_len: usize) -> SampleBatchResult {
        let request = ReplayRequest::SampleBatch { batch_len };
        let stream = match TcpStream::connect("localhost:43430") {
            Ok(stream) => stream,
            Err(e) => {
                panic!("Could not connect to replay server: {}", e);
            }
        };
        bincode::serialize_into(&stream, &request).unwrap();
        bincode::deserialize_from(stream).unwrap()
    }
    pub fn save<P: AsRef<Path>>(&self, _path: P) {
        todo!()
    }
    pub fn load<P: AsRef<Path>>(&mut self, _path: P) {
        todo!()
    }
}
