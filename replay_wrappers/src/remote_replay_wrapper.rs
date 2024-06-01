use model::traits::{Actor, ParamFetcher, Persistable, PrioritizedLearner, TargetNet};
use model::{BasicModel, LearningStepInfo, Params};
use packets::{PriorityUpdate, SampleBatchErrorKind, SampleBatchReply, SampleBatchResult};
use replay_data::CompressedTransition;
use replay_memories::ReplayRemote;
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

    fn convert_abs_td_error_to_priority(&self, abs_td_error: f64) -> f64 {
        const EPSILON: f64 = 0.001;
        (abs_td_error + EPSILON).powf(self.alpha)
    }

    fn update_priorities_from_td_errors(&mut self, indices: &[usize], abs_td_errors: &[f64]) {
        let priorities = abs_td_errors
            .into_iter()
            .map(|abs_td_error| self.convert_abs_td_error_to_priority(*abs_td_error));
        let updates = priorities
            .zip(indices.into_iter())
            .map(|(priority, index)| PriorityUpdate {
                index: *index,
                priority,
            });
        self.memory.update_priorities(updates.collect());
    }
}

impl RemoteReplayWrapper<BasicModel> {
    fn compute_priority(&self, transition: &CompressedTransition) -> f64 {
        let abs_td_error: f64 = self.model.compute_abs_td_errors(&[transition])[0].into();
        self.convert_abs_td_error_to_priority(abs_td_error)
    }

    pub fn remember(&mut self, transition: CompressedTransition) {
        let priority = self.compute_priority(&transition);
        self.memory
            .add_transition_with_priority(transition, priority);
    }
}

impl<T: Actor<State>, State> Actor<State> for RemoteReplayWrapper<T> {
    fn best_action(&self, state: &State) -> u8 {
        self.model.best_action(state)
    }
}

impl<T: PrioritizedLearner<CompressedTransition>> RemoteReplayWrapper<T> {
    fn train_on_sampled_batch(&mut self, reply: SampleBatchReply, beta: f64) -> LearningStepInfo {
        let SampleBatchReply {
            batch,
            min_probability,
            replay_len,
        } = reply;
        let (indices, probabilities, transitions) = batch;
        let (step_info, abs_td_errors) = self.model.train_batch_prioritized(
            &transitions.iter().collect::<Vec<_>>(),
            &probabilities,
            min_probability,
            replay_len,
            beta,
        );
        self.update_priorities_from_td_errors(&indices, &abs_td_errors);
        step_info
    }

    pub fn train_step(&mut self, beta: f64) -> Option<LearningStepInfo> {
        const BATCH_SIZE: usize = 32;
        match self.memory.sample_batch(BATCH_SIZE) {
            SampleBatchResult::Ok(reply) => Some(self.train_on_sampled_batch(reply, beta)),
            SampleBatchResult::Err(err) => match err {
                SampleBatchErrorKind::NotEnoughTransitions => None,
            },
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

impl<T: ParamFetcher> ParamFetcher for RemoteReplayWrapper<T> {
    fn params(&self) -> Params {
        self.model.params()
    }

    fn set_params(&mut self, params: Params) {
        self.model.set_params(params)
    }
}
