use super::LearningStepInfo;
use replay_data::{CompressedRcState, CompressedRcTransition};
use std::path::Path;

pub trait Actor {
    fn best_action(&self, state: &CompressedRcState) -> u8;
}

pub trait BasicLearner {
    fn train_batch(&mut self, batch: &[&CompressedRcTransition]) -> LearningStepInfo;
}

pub trait PrioritizedLearner {
    fn train_batch_prioritized(
        &mut self,
        batch_transitions: &[&CompressedRcTransition],
        batch_probabilities: &[f64],
        min_probability: f64,
        replay_memory_len: usize,
        beta: f64,
    ) -> (LearningStepInfo, Vec<f64>);
}

pub trait TargetNet {
    fn copy_control_to_target(&mut self);
}

pub trait Persistable {
    fn save<P: AsRef<Path>>(&self, filepath: P);
    fn load<P: AsRef<Path>>(&mut self, filepath: P);
}
