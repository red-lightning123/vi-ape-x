mod model_fns;
mod to_pixels;

use super::traits::{
    Actor, BasicLearner, ParamFetcher, Persistable, PrioritizedLearner, TargetNet,
};
use super::LearningStepInfo;
use crate::Params;
use model_fns::ModelFns;
use replay_data::GenericTransition;
use std::path::Path;
use tensorflow::{Graph, SavedModelBundle, SessionOptions, Tensor};
use to_pixels::ToPixels;

pub struct BasicModel {
    model_bundle: SavedModelBundle,
    fns: ModelFns,
}

impl BasicModel {
    pub fn new<P: AsRef<Path>>(def_path: P) -> Self {
        let mut graph = Graph::new();
        let model_bundle =
            SavedModelBundle::load(&SessionOptions::new(), ["serve"], &mut graph, def_path)
                .expect("Couldn't load model");
        let fns = ModelFns::new(&model_bundle, &graph);
        Self { model_bundle, fns }
    }

    pub fn compute_abs_td_errors<State>(&self, batch: &[&GenericTransition<State>]) -> Vec<f32>
    where
        State: ToPixels,
    {
        let batch_len = batch.len();
        let mut states = Vec::with_capacity(batch_len * 8 * 72 * 128);
        let mut next_states = Vec::with_capacity(batch_len * 8 * 72 * 128);
        let mut actions = Vec::with_capacity(batch_len);
        let mut rewards = Vec::with_capacity(batch_len);
        let mut dones = Vec::with_capacity(batch_len);
        for transition in batch {
            states.extend(transition.state.to_pixels());
            next_states.extend(transition.next_state.to_pixels());
            actions.push(transition.action);
            rewards.push(transition.reward as f32);
            dones.push(f32::from(u8::from(transition.terminated)));
        }

        let batch_len = batch_len.try_into().unwrap();
        let states_arg = Tensor::new(&[batch_len, 8, 72, 128])
            .with_values(&states)
            .unwrap();
        let next_states_arg = Tensor::new(&[batch_len, 8, 72, 128])
            .with_values(&next_states)
            .unwrap();
        let actions_arg = Tensor::new(&[batch_len]).with_values(&actions).unwrap();
        let rewards_arg = Tensor::new(&[batch_len]).with_values(&rewards).unwrap();
        let dones_arg = Tensor::new(&[batch_len]).with_values(&dones).unwrap();

        let (abs_td_errors,): (Tensor<f32>,) = self.fns.compute_abs_td_errors.call(
            &self.model_bundle.session,
            (
                states_arg,
                next_states_arg,
                actions_arg,
                rewards_arg,
                dones_arg,
            ),
        );
        abs_td_errors.to_vec()
    }
}

impl<State> Actor<State> for BasicModel
where
    State: ToPixels,
{
    fn best_action(&self, state: &State) -> u8 {
        let state_values = state.to_pixels();
        let state_arg = Tensor::new(&[8, 72, 128])
            .with_values(&state_values)
            .unwrap();

        let (action,): (Tensor<i64>,) = self
            .fns
            .best_action
            .call(&self.model_bundle.session, (state_arg,));
        action.get(&[]) as u8
    }
}

impl<State> BasicLearner<GenericTransition<State>> for BasicModel
where
    State: ToPixels,
{
    fn train_batch(&mut self, batch: &[&GenericTransition<State>]) -> LearningStepInfo {
        let batch_len = batch.len();
        let mut states = Vec::with_capacity(batch_len * 8 * 72 * 128);
        let mut next_states = Vec::with_capacity(batch_len * 8 * 72 * 128);
        let mut actions = Vec::with_capacity(batch_len);
        let mut rewards = Vec::with_capacity(batch_len);
        let mut dones = Vec::with_capacity(batch_len);
        for transition in batch {
            states.extend(transition.state.to_pixels());
            next_states.extend(transition.next_state.to_pixels());
            actions.push(transition.action);
            rewards.push(transition.reward as f32);
            dones.push(f32::from(u8::from(transition.terminated)));
        }

        let batch_len = batch_len.try_into().unwrap();
        let states_arg = Tensor::new(&[batch_len, 8, 72, 128])
            .with_values(&states)
            .unwrap();
        let next_states_arg = Tensor::new(&[batch_len, 8, 72, 128])
            .with_values(&next_states)
            .unwrap();
        let actions_arg = Tensor::new(&[batch_len]).with_values(&actions).unwrap();
        let rewards_arg = Tensor::new(&[batch_len]).with_values(&rewards).unwrap();
        let dones_arg = Tensor::new(&[batch_len]).with_values(&dones).unwrap();

        let (loss, average_q_val): (Tensor<f32>, Tensor<f32>) = self.fns.train_batch.call(
            &self.model_bundle.session,
            (
                states_arg,
                next_states_arg,
                actions_arg,
                rewards_arg,
                dones_arg,
            ),
        );
        LearningStepInfo {
            loss: loss.get(&[]),
            average_q_val: average_q_val.get(&[]),
        }
    }
}

impl<State> PrioritizedLearner<GenericTransition<State>> for BasicModel
where
    State: ToPixels,
{
    fn train_batch_prioritized(
        &mut self,
        batch_transitions: &[&GenericTransition<State>],
        batch_probabilities: &[f64],
        min_probability: f64,
        replay_memory_len: usize,
        beta: f64,
    ) -> (LearningStepInfo, Vec<f64>) {
        let batch_len = batch_transitions.len();
        let mut states = Vec::with_capacity(batch_len * 8 * 72 * 128);
        let mut next_states = Vec::with_capacity(batch_len * 8 * 72 * 128);
        let mut actions = Vec::with_capacity(batch_len);
        let mut rewards = Vec::with_capacity(batch_len);
        let mut dones = Vec::with_capacity(batch_len);
        for transition in batch_transitions {
            states.extend(transition.state.to_pixels());
            next_states.extend(transition.next_state.to_pixels());
            actions.push(transition.action);
            rewards.push(transition.reward as f32);
            dones.push(f32::from(u8::from(transition.terminated)));
        }

        let batch_len = batch_len.try_into().unwrap();
        let states_arg = Tensor::new(&[batch_len, 8, 72, 128])
            .with_values(&states)
            .unwrap();
        let next_states_arg = Tensor::new(&[batch_len, 8, 72, 128])
            .with_values(&next_states)
            .unwrap();
        let actions_arg = Tensor::new(&[batch_len]).with_values(&actions).unwrap();
        let rewards_arg = Tensor::new(&[batch_len]).with_values(&rewards).unwrap();
        let dones_arg = Tensor::new(&[batch_len]).with_values(&dones).unwrap();
        let probabilities_arg = Tensor::new(&[batch_len])
            .with_values(
                &batch_probabilities
                    .iter()
                    .map(|x| *x as f32)
                    .collect::<Vec<_>>(),
            )
            .unwrap();
        let min_probability_arg = Tensor::new(&[])
            .with_values(&[min_probability as f32])
            .unwrap();
        let replay_memory_len_arg = Tensor::new(&[])
            .with_values(&[replay_memory_len as f32])
            .unwrap();
        let beta_arg = Tensor::new(&[]).with_values(&[beta as f32]).unwrap();

        let args = (
            states_arg,
            next_states_arg,
            actions_arg,
            rewards_arg,
            dones_arg,
            probabilities_arg,
            min_probability_arg,
            replay_memory_len_arg,
            beta_arg,
        );

        let (loss, average_q_val, abs_td_errors): (Tensor<f32>, Tensor<f32>, Tensor<f32>) = self
            .fns
            .train_batch_prioritized
            .call(&self.model_bundle.session, args);
        let learning_step_info = LearningStepInfo {
            loss: loss.get(&[]),
            average_q_val: average_q_val.get(&[]),
        };
        (
            learning_step_info,
            abs_td_errors.iter().map(|x| *x as f64).collect::<Vec<_>>(),
        )
    }
}

impl TargetNet for BasicModel {
    fn copy_control_to_target(&mut self) {
        let (_,): (Tensor<i32>,) = self
            .fns
            .copy_control_to_target
            .call(&self.model_bundle.session, ());
    }
}

impl BasicModel {
    fn save_internal(&self, filepath: String) {
        let path_arg = Tensor::new(&[]).with_values(&[filepath]).unwrap();
        let (_,): (Tensor<i32>,) = self.fns.save.call(&self.model_bundle.session, (path_arg,));
    }
    fn load_internal(&self, filepath: String) {
        let path_arg = Tensor::new(&[]).with_values(&[filepath]).unwrap();
        let (_,): (Tensor<i32>,) = self.fns.load.call(&self.model_bundle.session, (path_arg,));
    }
}

impl Persistable for BasicModel {
    fn save<P: AsRef<Path>>(&self, path: P) {
        self.save_internal(path.as_ref().to_str().unwrap().to_string());
    }
    fn load<P: AsRef<Path>>(&mut self, path: P) {
        self.load_internal(path.as_ref().to_str().unwrap().to_string());
    }
}

impl ParamFetcher for BasicModel {
    fn params(&self) -> Params {
        let (params,): (Tensor<String>,) = self.fns.params.call(&self.model_bundle.session, ());
        Params(params.to_vec())
    }
    fn set_params(&mut self, params: Params) {
        let params = params.0;
        let params = Tensor::new(&[params.len() as u64])
            .with_values(&params)
            .unwrap();
        let (_,): (Tensor<i32>,) = self
            .fns
            .set_params
            .call(&self.model_bundle.session, (params,));
    }
}
