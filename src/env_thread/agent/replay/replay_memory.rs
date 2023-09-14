use crate::env_thread::Transition;
use std::path::Path;

pub trait ReplayMemory {
    fn with_max_size(max_size : usize) -> Self;
    fn add_transition(&mut self, transition : Transition);
    fn sample_batch(&self, batch_size : usize) -> Vec<&Transition>;
    fn len(&self) -> usize;
    fn save<P : AsRef<Path>>(&self, path : P);
    fn load<P : AsRef<Path>>(&mut self, path : P);
}

