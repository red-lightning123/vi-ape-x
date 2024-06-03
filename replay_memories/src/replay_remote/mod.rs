mod replay_client;

use packets::{Insertion, PriorityUpdate, SampleBatchResult};
use replay_client::ReplayClient;
use replay_data::CompressedTransition;
use std::mem;
use std::net::SocketAddr;
use std::path::Path;

pub struct ReplayRemote {
    insertion_batch: Vec<Insertion>,
    client: ReplayClient,
}

impl ReplayRemote {
    pub fn new(replay_server_addr: SocketAddr) -> Self {
        Self {
            insertion_batch: vec![],
            client: ReplayClient::new(replay_server_addr),
        }
    }
    pub fn update_priorities(&mut self, batch: Vec<PriorityUpdate>) {
        self.client.update_priorities(batch)
    }
    pub fn add_transition_with_priority(
        &mut self,
        transition: CompressedTransition,
        priority: f64,
    ) {
        const INSERTION_BATCH_LEN: usize = 50;
        let insertion = Insertion {
            priority,
            transition,
        };
        self.insertion_batch.push(insertion);
        if self.insertion_batch.len() >= INSERTION_BATCH_LEN {
            let insertion_batch = mem::replace(&mut self.insertion_batch, vec![]);
            self.client.insert(insertion_batch);
        }
    }
    pub fn sample_batch(&self, batch_len: usize) -> SampleBatchResult {
        self.client.sample_batch(batch_len)
    }
    pub fn save<P: AsRef<Path>>(&self, _path: P) {
        todo!()
    }
    pub fn load<P: AsRef<Path>>(&mut self, _path: P) {
        todo!()
    }
}
