use super::transition_saving::{load_transitions, save_transitions};
use crate::file_io::{create_file_buf_write, open_file_buf_read};
use rand::prelude::{IteratorRandom, SliceRandom};
use replay_data::CompressedTransition;
use std::collections::VecDeque;
use std::path::Path;

pub struct ReplayQueue {
    transitions: VecDeque<CompressedTransition>,
    max_size: usize,
}

impl ReplayQueue {
    pub fn with_max_size(max_size: usize) -> Self {
        Self {
            transitions: VecDeque::with_capacity(max_size),
            max_size,
        }
    }
    pub fn add_transition(&mut self, transition: CompressedTransition) {
        if self.transitions.len() >= self.max_size {
            self.transitions.pop_front();
        }
        self.transitions.push_back(transition);
    }
    pub fn sample_batch(&self, batch_size: usize) -> Vec<&CompressedTransition> {
        let mut batch = self
            .transitions
            .iter()
            .choose_multiple(&mut rand::thread_rng(), batch_size);
        batch.shuffle(&mut rand::thread_rng());
        batch
    }
    pub fn len(&self) -> usize {
        self.transitions.len()
    }
    pub fn save<P: AsRef<Path>>(&self, path: P) {
        let path = path.as_ref();
        let max_size_file = create_file_buf_write(path.join("max_size")).unwrap();
        bincode::serialize_into(max_size_file, &self.max_size).unwrap();
        save_transitions(path, &self.transitions);
    }
    pub fn load<P: AsRef<Path>>(&mut self, path: P) {
        let path = path.as_ref();
        let max_size_file = open_file_buf_read(path.join("max_size")).unwrap();
        self.max_size = bincode::deserialize_from(max_size_file).unwrap();
        let transitions = load_transitions(path, self.max_size);
        self.transitions = transitions.into();
    }
}
