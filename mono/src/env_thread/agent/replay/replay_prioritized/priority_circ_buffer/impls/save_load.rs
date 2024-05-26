use super::transition_saving::{load_transitions, save_transitions};
use super::PriorityCircBuffer;
use crate::file_io::{create_file_buf_write, open_file_buf_read};
use replay_data::CompressedRcTransition;
use std::path::Path;

impl PriorityCircBuffer<f64, CompressedRcTransition> {
    pub fn save<P: AsRef<Path>>(&self, path: P) {
        let path = path.as_ref();
        let max_size_file = create_file_buf_write(path.join("max_size")).unwrap();
        bincode::serialize_into(max_size_file, &self.max_size).unwrap();
        save_transitions(path, &self.values);
        let head_file = create_file_buf_write(path.join("head")).unwrap();
        bincode::serialize_into(head_file, &self.head).unwrap();
        let priorities_file = create_file_buf_write(path.join("priorities")).unwrap();
        bincode::serialize_into(priorities_file, &self.priorities).unwrap();
    }
    pub fn load<P: AsRef<Path>>(&mut self, path: P) {
        let path = path.as_ref();
        let max_size_file = open_file_buf_read(path.join("max_size")).unwrap();
        self.max_size = bincode::deserialize_from(max_size_file).unwrap();
        self.values = load_transitions(path, self.max_size);
        let head_file = open_file_buf_read(path.join("head")).unwrap();
        self.head = bincode::deserialize_from(head_file).unwrap();
        let priorities_file = open_file_buf_read(path.join("priorities")).unwrap();
        self.priorities = bincode::deserialize_from(priorities_file).unwrap();
    }
}
