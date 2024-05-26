use super::replay::ReplayQueue;
use super::traits::{Actor, BasicLearner, Persistable, TargetNet};
use super::LearningStepInfo;
use replay_data::{CompressedState, CompressedTransition};
use std::fs;
use std::path::Path;

pub struct QueueReplayWrapper<T> {
    model: T,
    memory: ReplayQueue,
}

impl<T> QueueReplayWrapper<T> {
    pub fn wrap(model: T, memory_capacity: usize) -> Self {
        Self {
            model,
            memory: ReplayQueue::with_max_size(memory_capacity),
        }
    }
    pub fn remember(&mut self, transition: CompressedTransition) {
        self.memory.add_transition(transition);
    }
}

impl<T: Actor> Actor for QueueReplayWrapper<T> {
    fn best_action(&self, state: &CompressedState) -> u8 {
        self.model.best_action(state)
    }
}

impl<T: BasicLearner> QueueReplayWrapper<T> {
    pub fn train_step(&mut self) -> Option<LearningStepInfo> {
        const BATCH_SIZE: usize = 32;
        if self.memory.len() >= BATCH_SIZE {
            let batch_transitions = self.memory.sample_batch(BATCH_SIZE);
            let step_info = self.model.train_batch(&batch_transitions);
            Some(step_info)
        } else {
            None
        }
    }
}

impl<T: TargetNet> TargetNet for QueueReplayWrapper<T> {
    fn copy_control_to_target(&mut self) {
        self.model.copy_control_to_target();
    }
}

impl<T: Persistable> Persistable for QueueReplayWrapper<T> {
    fn save<P: AsRef<Path>>(&self, path: P) {
        let path = path.as_ref();
        self.model.save(path.join("model_vars").to_str().unwrap());
        let memory_path = path.join("memory");
        fs::create_dir_all(&memory_path).unwrap();
        self.memory.save(memory_path);
    }
    fn load<P: AsRef<Path>>(&mut self, path: P) {
        let path = path.as_ref();
        self.model.load(path.join("model_vars").to_str().unwrap());
        self.memory.load(path.join("memory"));
    }
}
