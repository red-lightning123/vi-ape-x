use super::serialize_transitions::{frames_transitions_serialized, values_deserialized};
use crate::env_thread::agent::Transition;
use crate::file_io::{create_file_buf_write, open_file_buf_read};
use rand::prelude::{IteratorRandom, SliceRandom};
use std::collections::VecDeque;
use std::path::Path;

pub struct ReplayQueue {
    transitions: VecDeque<Transition>,
    max_size: usize,
}

impl ReplayQueue {
    pub fn with_max_size(max_size: usize) -> Self {
        Self {
            transitions: VecDeque::with_capacity(max_size),
            max_size,
        }
    }
    pub fn add_transition(&mut self, transition: Transition) {
        if self.transitions.len() >= self.max_size {
            self.transitions.pop_front();
        }
        self.transitions.push_back(transition);
    }
    pub fn sample_batch(&self, batch_size: usize) -> Vec<&Transition> {
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
        // the experience replay queue can take up a lot of space, therefore we serialize each
        // frame/transition separately in a streaming manner so as to not inadvertently clone
        // the entire queue (which would cause a spike in RAM usage and might result in OOM)
        let max_size_file = create_file_buf_write(path.as_ref().join("max_size")).unwrap();
        bincode::serialize_into(max_size_file, &self.max_size).unwrap();
        let (serialized_frames, serialized_transitions) =
            frames_transitions_serialized(&self.transitions);
        let mut frames_file = create_file_buf_write(path.as_ref().join("frames")).unwrap();
        for frame in serialized_frames {
            bincode::serialize_into(&mut frames_file, &**frame).unwrap();
        }
        let mut transitions_file =
            create_file_buf_write(path.as_ref().join("transitions")).unwrap();
        for transition in serialized_transitions {
            bincode::serialize_into(&mut transitions_file, &transition).unwrap();
        }
    }
    pub fn load<P: AsRef<Path>>(&mut self, path: P) {
        let max_size_file = open_file_buf_read(path.as_ref().join("max_size")).unwrap();
        self.max_size = bincode::deserialize_from(max_size_file).unwrap();
        let deserialized_values = values_deserialized(path.as_ref(), self.max_size);
        self.transitions = deserialized_values.into();
    }
}
