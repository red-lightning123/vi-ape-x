use super::{State, Transition};
use std::fs;
use std::path::Path;
pub mod replay;
use replay::ReplayMemory;
mod model;
use model::Model;

pub struct Agent<R: ReplayMemory> {
    model: Model,
    memory: R,
}

impl<R: ReplayMemory> Agent<R> {
    pub fn with_memory_capacity(memory_capacity: usize) -> Agent<R> {
        Agent {
            model: Model::new(),
            memory: R::with_max_size(memory_capacity),
        }
    }
    pub fn best_action(&self, state: &State) -> u8 {
        self.model.best_action(state)
    }
    pub fn remember(&mut self, transition: Transition) {
        self.memory.add_transition(transition);
    }
    pub fn train_step(&mut self, beta: f64) -> Option<f32> {
        const BATCH_SIZE: usize = 32;
        if self.memory.len() >= BATCH_SIZE {
            // TODO: try to support both prioritized and non-prioritized
            // replay memory. for example, batch errors shouldn't be computed
            // when replay memory isn't prioritized
            let (batch_indices, batch_probabilities, batch_transitions) =
                self.memory.sample_batch_prioritized(BATCH_SIZE);
            let min_probability = self.memory.min_probability();
            let (loss, batch_abs_td_errors) = self.model.train_batch_prioritized(
                &batch_transitions,
                &batch_probabilities,
                min_probability,
                self.memory.len(),
                beta,
            );
            const ALPHA: f64 = 0.6;
            self.memory.update_priorities_with_td_errors(
                &batch_indices,
                &batch_abs_td_errors,
                ALPHA,
            );
            Some(loss)
        } else {
            None
        }
    }
    pub fn copy_control_to_target(&mut self) {
        self.model.copy_control_to_target();
    }
    pub fn save<P: AsRef<Path>>(&self, path: P) {
        let path = path.as_ref();
        self.model.save(path.join("model_vars").to_str().unwrap());
        let memory_path = path.join("memory");
        fs::create_dir_all(&memory_path).unwrap();
        self.memory.save(memory_path);
    }
    pub fn load<P: AsRef<Path>>(&mut self, path: P) {
        let path = path.as_ref();
        self.model.load(path.join("model_vars").to_str().unwrap());
        self.memory.load(path.join("memory"));
    }
}
