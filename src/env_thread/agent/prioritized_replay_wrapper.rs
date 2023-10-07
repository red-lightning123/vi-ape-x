use super::replay::ReplayPrioritized;
use super::Model;
use super::{State, Transition};
use std::fs;
use std::path::Path;

pub struct PrioritizedReplayWrapper<T> {
    model: T,
    memory: ReplayPrioritized,
    alpha: f64,
}

impl<T: Model> PrioritizedReplayWrapper<T> {
    pub fn wrap(model: T, memory_capacity: usize, alpha: f64) -> PrioritizedReplayWrapper<T> {
        PrioritizedReplayWrapper {
            model,
            memory: ReplayPrioritized::with_max_size(memory_capacity),
            alpha,
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
            let (batch_indices, batch_probabilities, batch_transitions) =
                self.memory.sample_batch(BATCH_SIZE);
            let min_probability = self.memory.min_probability();
            let (loss, batch_abs_td_errors) = self.model.train_batch_prioritized(
                &batch_transitions,
                &batch_probabilities,
                min_probability,
                self.memory.len(),
                beta,
            );
            self.memory.update_priorities_with_td_errors(
                &batch_indices,
                &batch_abs_td_errors,
                self.alpha,
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
