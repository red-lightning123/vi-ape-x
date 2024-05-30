mod model_fns;

use super::traits::{
    Actor, BasicLearner, ParamFetcher, Persistable, PrioritizedLearner, TargetNet,
};
use super::LearningStepInfo;
use crate::Params;
use image::{ImageOwned, ImageRef2};
use model_fns::ModelFns;
use replay_data::{CompressedRcState, CompressedRcTransition, State};
use std::path::Path;
use tensorflow::{Graph, SavedModelBundle, SessionOptions, Tensor};

fn extract_planes(frame: &ImageRef2) -> (Vec<u8>, Vec<u8>) {
    frame.data().chunks(2).map(|a| (a[0], a[1])).unzip()
}

fn frame_to_pixels(frame: &ImageRef2) -> Vec<u8> {
    let (plane_0, plane_1) = extract_planes(frame);
    [plane_0, plane_1].concat()
}

fn state_to_pixels(state: &CompressedRcState) -> Vec<u8> {
    let state: State = state.into();
    state
        .frames()
        .iter()
        .map(|frame| frame.as_ref())
        .map(|frame| frame_to_pixels(&frame))
        .collect::<Vec<_>>()
        .concat()
}

pub struct BasicModel {
    model_bundle: SavedModelBundle,
    fns: ModelFns,
}

impl BasicModel {
    pub fn new() -> Self {
        let mut graph = Graph::new();
        let model_bundle =
            SavedModelBundle::load(&SessionOptions::new(), ["serve"], &mut graph, "model")
                .expect("Couldn't load model");
        let fns = ModelFns::new(&model_bundle, &graph);
        Self { model_bundle, fns }
    }
}

impl Actor for BasicModel {
    fn best_action(&self, state: &CompressedRcState) -> u8 {
        let state_values = state_to_pixels(state);
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

impl BasicLearner for BasicModel {
    fn train_batch(&mut self, batch: &[&CompressedRcTransition]) -> LearningStepInfo {
        let batch_len = batch.len();
        let mut states = Vec::with_capacity(batch_len * 8 * 72 * 128);
        let mut next_states = Vec::with_capacity(batch_len * 8 * 72 * 128);
        let mut actions = Vec::with_capacity(batch_len);
        let mut rewards = Vec::with_capacity(batch_len);
        let mut dones = Vec::with_capacity(batch_len);
        for transition in batch {
            states.extend(state_to_pixels(&transition.state));
            next_states.extend(state_to_pixels(&transition.next_state));
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

impl PrioritizedLearner for BasicModel {
    fn train_batch_prioritized(
        &mut self,
        batch_transitions: &[&CompressedRcTransition],
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
            states.extend(state_to_pixels(&transition.state));
            next_states.extend(state_to_pixels(&transition.next_state));
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
