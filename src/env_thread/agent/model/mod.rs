use super::{State, Transition};
mod model_fns;
use crate::ImageOwned;
use model_fns::ModelFns;
use tensorflow::{Graph, SavedModelBundle, SessionOptions, Tensor};

fn state_to_pixels(state: &State) -> Vec<u8> {
    [
        (*state[0]).as_ref().data(),
        (*state[1]).as_ref().data(),
        (*state[2]).as_ref().data(),
        (*state[3]).as_ref().data(),
    ]
    .concat()
}

pub struct Model {
    model_bundle: SavedModelBundle,
    fns: ModelFns,
}

impl Model {
    pub fn new() -> Model {
        let mut graph = Graph::new();
        let model_bundle =
            SavedModelBundle::load(&SessionOptions::new(), ["serve"], &mut graph, "model")
                .expect("Couldn't load model");
        let fns = ModelFns::new(&model_bundle, &graph);
        Model { model_bundle, fns }
    }
    pub fn best_action(&self, state: &State) -> u8 {
        let state_values = state_to_pixels(state);
        let state_arg = Tensor::new(&[8, 128, 72])
            .with_values(&state_values)
            .unwrap();

        let (action,): (Tensor<i64>,) = self
            .fns
            .best_action
            .call(&self.model_bundle.session, (state_arg,));
        action.get(&[]) as u8
    }
    pub fn train_batch(&mut self, batch: &[&Transition]) -> f32 {
        let mut states = Vec::with_capacity(32 * 8 * 128 * 72);
        let mut next_states = Vec::with_capacity(32 * 8 * 128 * 72);
        let mut actions = Vec::with_capacity(32);
        let mut rewards = Vec::with_capacity(32);
        let mut dones = Vec::with_capacity(32);
        for (state, next_state, action, reward, terminated) in batch {
            states.extend(state_to_pixels(state));
            next_states.extend(state_to_pixels(next_state));
            actions.push(*action);
            rewards.push(*reward as f32);
            dones.push(f32::from(u8::from(*terminated)));
        }

        let states_arg = Tensor::new(&[32, 8, 128, 72]).with_values(&states).unwrap();
        let next_states_arg = Tensor::new(&[32, 8, 128, 72])
            .with_values(&next_states)
            .unwrap();
        let actions_arg = Tensor::new(&[32]).with_values(&actions).unwrap();
        let rewards_arg = Tensor::new(&[32]).with_values(&rewards).unwrap();
        let dones_arg = Tensor::new(&[32]).with_values(&dones).unwrap();

        let (loss,): (Tensor<f32>,) = self.fns.train_batch.call(
            &self.model_bundle.session,
            (
                states_arg,
                next_states_arg,
                actions_arg,
                rewards_arg,
                dones_arg,
            ),
        );
        loss.get(&[])
    }
    pub fn train_batch_prioritized(
        &mut self,
        batch_transitions: &[&Transition],
        batch_probabilities: &[f64],
        min_probability: f64,
        replay_memory_len: usize,
        beta: f64,
    ) -> (f32, Vec<f64>) {
        let mut states = Vec::with_capacity(32 * 8 * 128 * 72);
        let mut next_states = Vec::with_capacity(32 * 8 * 128 * 72);
        let mut actions = Vec::with_capacity(32);
        let mut rewards = Vec::with_capacity(32);
        let mut dones = Vec::with_capacity(32);
        for (state, next_state, action, reward, terminated) in batch_transitions {
            states.extend(state_to_pixels(state));
            next_states.extend(state_to_pixels(next_state));
            actions.push(*action);
            rewards.push(*reward as f32);
            dones.push(f32::from(u8::from(*terminated)));
        }

        let states_arg = Tensor::new(&[32, 8, 128, 72]).with_values(&states).unwrap();
        let next_states_arg = Tensor::new(&[32, 8, 128, 72])
            .with_values(&next_states)
            .unwrap();
        let actions_arg = Tensor::new(&[32]).with_values(&actions).unwrap();
        let rewards_arg = Tensor::new(&[32]).with_values(&rewards).unwrap();
        let dones_arg = Tensor::new(&[32]).with_values(&dones).unwrap();
        let probabilities_arg = Tensor::new(&[32])
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

        let (loss, abs_td_errors): (Tensor<f32>, Tensor<f32>) = self
            .fns
            .train_batch_prioritized
            .call(&self.model_bundle.session, args);
        (
            loss.get(&[]),
            abs_td_errors.iter().map(|x| *x as f64).collect::<Vec<_>>(),
        )
    }
    pub fn copy_control_to_target(&mut self) {
        let (_,): (Tensor<i32>,) = self
            .fns
            .copy_control_to_target
            .call(&self.model_bundle.session, ());
    }
    pub fn save(&self, filepath: &str) {
        let path_arg = Tensor::new(&[])
            .with_values(&[filepath.to_string()])
            .unwrap();
        let (_,): (Tensor<i32>,) = self.fns.save.call(&self.model_bundle.session, (path_arg,));
    }
    pub fn load(&self, filepath: &str) {
        let path_arg = Tensor::new(&[])
            .with_values(&[filepath.to_string()])
            .unwrap();
        let (_,): (Tensor<i32>,) = self.fns.load.call(&self.model_bundle.session, (path_arg,));
    }
}
