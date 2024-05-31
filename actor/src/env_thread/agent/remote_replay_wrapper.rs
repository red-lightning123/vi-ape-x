use super::replay::ReplayRemote;
use model::traits::{Actor, Persistable, PrioritizedLearner, TargetNet};
use model::LearningStepInfo;
use replay_data::{CompressedRcState, CompressedRcTransition};
use std::fs;
use std::path::Path;

pub struct RemoteReplayWrapper<T> {
    model: T,
    memory: ReplayRemote,
    alpha: f64,
}

impl<T> RemoteReplayWrapper<T> {
    pub fn wrap(model: T, alpha: f64) -> Self {
        Self {
            model,
            memory: ReplayRemote::new(),
            alpha,
        }
    }
    pub fn remember(&mut self, transition: CompressedRcTransition) {
        self.memory
            .add_transition(todo!("convert transition to CompressedTransition"));
    }
}

impl<T: Actor> Actor for RemoteReplayWrapper<T> {
    fn best_action(&self, state: &CompressedRcState) -> u8 {
        self.model
            .best_action(todo!("convert state to CompressedState"))
    }
}

impl<T: PrioritizedLearner> RemoteReplayWrapper<T> {
    pub fn train_step(&mut self, beta: f64) -> Option<LearningStepInfo> {
        const BATCH_SIZE: usize = 32;
        if self.memory.len() >= BATCH_SIZE {
            let (batch_indices, batch_probabilities, batch_transitions) =
                self.memory.sample_batch(BATCH_SIZE);
            let min_probability = self.memory.min_probability();
            let (step_info, batch_abs_td_errors) = self.model.train_batch_prioritized(
                todo!("convert batch_transitions to CompressedRcTransitions"),
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
            Some(step_info)
        } else {
            None
        }
    }
}

impl<T: TargetNet> TargetNet for RemoteReplayWrapper<T> {
    fn copy_control_to_target(&mut self) {
        self.model.copy_control_to_target();
    }
}

impl<T: Persistable> Persistable for RemoteReplayWrapper<T> {
    fn save<P: AsRef<Path>>(&self, path: P) {
        let path = path.as_ref();
        self.model.save(path.join("model_vars"));
        let memory_path = path.join("memory");
        fs::create_dir_all(&memory_path).unwrap();
        self.memory.save(memory_path);
    }
    fn load<P: AsRef<Path>>(&mut self, path: P) {
        let path = path.as_ref();
        self.model.load(path.join("model_vars"));
        self.memory.load(path.join("memory"));
    }
}
