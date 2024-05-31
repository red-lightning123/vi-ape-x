use super::{LearningStepInfo, Params};
use std::path::Path;

pub trait Actor<State> {
    fn best_action(&self, state: &State) -> u8;
}

pub trait BasicLearner<Transition> {
    fn train_batch(&mut self, batch: &[&Transition]) -> LearningStepInfo;
}

pub trait PrioritizedLearner<Transition> {
    fn train_batch_prioritized(
        &mut self,
        batch_transitions: &[&Transition],
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

pub trait ParamFetcher {
    fn params(&self) -> Params;
    fn set_params(&mut self, params: Params);
}
