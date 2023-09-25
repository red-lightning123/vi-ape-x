use super::{ State, Transition };
use crate::ImageOwned;
use tensorflow::{ Graph, SavedModelBundle, SessionOptions, SessionRunArgs, Tensor };

fn state_to_pixels(state : &State) -> Vec<u8> {
    [(*state[0]).as_ref().data(), (*state[1]).as_ref().data(), (*state[2]).as_ref().data(), (*state[3]).as_ref().data()].concat()
}

pub struct Model {
    model_bundle : SavedModelBundle,
    graph : Graph
}

impl Model {
    pub fn new() -> Model {
        let mut graph = Graph::new();
        let model_bundle = SavedModelBundle::load(&SessionOptions::new(), ["serve"], &mut graph, "model").expect("Couldn't load model");
        Model {
            model_bundle,
            graph
        }
    }
    pub fn best_action(&self, state : &State) -> u8 {
        let session = &self.model_bundle.session;
        let signature = self.model_bundle.meta_graph_def().get_signature("best_action").unwrap();

        let state_arg_info = signature.get_input("state").unwrap();
        let output_info = signature.get_output("output_0").unwrap();

        let state_arg_op = self.graph.operation_by_name_required(&state_arg_info.name().name).unwrap();
        let output_op = self.graph.operation_by_name_required(&output_info.name().name).unwrap();

        let state_values = state_to_pixels(state);
        let state_tensor = Tensor::new(&[8, 128, 72]).with_values(&state_values).unwrap();

        let mut args = SessionRunArgs::new();

        args.add_feed(&state_arg_op, 0, &state_tensor);

        let output_fetch_token = args.request_fetch(&output_op, 0);

        session
            .run(&mut args)
            .expect("best_action couldn't run session");

        let output_tensor : Tensor<i64> = args.fetch(output_fetch_token).unwrap();

        output_tensor.get(&[]) as u8
    }
    pub fn train_batch(&mut self, batch : &[&Transition]) -> f32 {
        let session = &self.model_bundle.session;
        let signature = self.model_bundle.meta_graph_def().get_signature("train_pred_step").unwrap();

        let states_arg_info = signature.get_input("states").unwrap();
        let next_states_arg_info = signature.get_input("new_states").unwrap();
        let actions_arg_info = signature.get_input("actions").unwrap();
        let rewards_arg_info = signature.get_input("rewards").unwrap();
        let dones_arg_info = signature.get_input("dones").unwrap();
        let output_info = signature.get_output("output_0").unwrap();
        
        let states_arg_op = self.graph.operation_by_name_required(&states_arg_info.name().name).unwrap();
        let next_states_arg_op = self.graph.operation_by_name_required(&next_states_arg_info.name().name).unwrap();
        let actions_arg_op = self.graph.operation_by_name_required(&actions_arg_info.name().name).unwrap();
        let rewards_arg_op = self.graph.operation_by_name_required(&rewards_arg_info.name().name).unwrap();
        let dones_arg_op = self.graph.operation_by_name_required(&dones_arg_info.name().name).unwrap();
        let output_op = self.graph.operation_by_name_required(&output_info.name().name).unwrap();
        
        let mut states = Vec::with_capacity(32*8*128*72);
        let mut next_states = Vec::with_capacity(32*8*128*72);
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
        
        let states_tensor = Tensor::new(&[32, 8, 128, 72]).with_values(&states).unwrap();
        let next_states_tensor = Tensor::new(&[32, 8, 128, 72]).with_values(&next_states).unwrap();
        let actions_tensor = Tensor::new(&[32]).with_values(&actions).unwrap();
        let rewards_tensor = Tensor::new(&[32]).with_values(&rewards).unwrap();
        let dones_tensor = Tensor::new(&[32]).with_values(&dones).unwrap();

        let mut args = SessionRunArgs::new();

        args.add_feed(&states_arg_op, 0, &states_tensor);
        args.add_feed(&next_states_arg_op, 0, &next_states_tensor);
        args.add_feed(&actions_arg_op, 0, &actions_tensor);
        args.add_feed(&rewards_arg_op, 0, &rewards_tensor);
        args.add_feed(&dones_arg_op, 0, &dones_tensor);

        let output_fetch_token = args.request_fetch(&output_op, 0);

        session
            .run(&mut args)
            .expect("train_batch couldn't run session");

        let output_tensor : Tensor<f32> = args.fetch(output_fetch_token).unwrap();

        output_tensor.get(&[])
    }
    pub fn train_batch_prioritized(&mut self, batch_transitions : &[&Transition], batch_probabilities : &[f64], min_probability : f64, replay_memory_len : usize, beta : f64) -> (f32, Vec<f64>) {
        let session = &self.model_bundle.session;
        let signature = self.model_bundle.meta_graph_def().get_signature("train_pred_step_prioritized").unwrap();

        let states_arg_info = signature.get_input("states").unwrap();
        let next_states_arg_info = signature.get_input("new_states").unwrap();
        let actions_arg_info = signature.get_input("actions").unwrap();
        let rewards_arg_info = signature.get_input("rewards").unwrap();
        let dones_arg_info = signature.get_input("dones").unwrap();
        let probabilities_arg_info = signature.get_input("probabilities").unwrap();
        let min_probability_arg_info = signature.get_input("min_probability").unwrap();
        let replay_memory_len_arg_info = signature.get_input("replay_memory_len").unwrap();
        let beta_arg_info = signature.get_input("beta").unwrap();
        let output_loss_info = signature.get_output("output_0").unwrap();
        let output_abs_td_errors_info = signature.get_output("output_1").unwrap();
        
        let states_arg_op = self.graph.operation_by_name_required(&states_arg_info.name().name).unwrap();
        let next_states_arg_op = self.graph.operation_by_name_required(&next_states_arg_info.name().name).unwrap();
        let actions_arg_op = self.graph.operation_by_name_required(&actions_arg_info.name().name).unwrap();
        let rewards_arg_op = self.graph.operation_by_name_required(&rewards_arg_info.name().name).unwrap();
        let dones_arg_op = self.graph.operation_by_name_required(&dones_arg_info.name().name).unwrap();
        let probabilities_arg_op = self.graph.operation_by_name_required(&probabilities_arg_info.name().name).unwrap();
        let min_probability_arg_op = self.graph.operation_by_name_required(&min_probability_arg_info.name().name).unwrap();
        let replay_memory_len_arg_op = self.graph.operation_by_name_required(&replay_memory_len_arg_info.name().name).unwrap();
        let beta_arg_op = self.graph.operation_by_name_required(&beta_arg_info.name().name).unwrap();
        let output_loss_op = self.graph.operation_by_name_required(&output_loss_info.name().name).unwrap();
        let output_abs_td_errors_op = self.graph.operation_by_name_required(&output_abs_td_errors_info.name().name).unwrap();
        
        let mut states = Vec::with_capacity(32*8*128*72);
        let mut next_states = Vec::with_capacity(32*8*128*72);
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
        
        let states_tensor = Tensor::new(&[32, 8, 128, 72]).with_values(&states).unwrap();
        let next_states_tensor = Tensor::new(&[32, 8, 128, 72]).with_values(&next_states).unwrap();
        let actions_tensor = Tensor::new(&[32]).with_values(&actions).unwrap();
        let rewards_tensor = Tensor::new(&[32]).with_values(&rewards).unwrap();
        let dones_tensor = Tensor::new(&[32]).with_values(&dones).unwrap();
        let probabilities_tensor = Tensor::new(&[32]).with_values(&batch_probabilities.iter().map(|x| *x as f32).collect::<Vec<_>>()).unwrap();
        let min_probability_tensor = Tensor::new(&[]).with_values(&[min_probability as f32]).unwrap();
        let replay_memory_len_tensor = Tensor::new(&[]).with_values(&[replay_memory_len as f32]).unwrap();
        let beta_tensor = Tensor::new(&[]).with_values(&[beta as f32]).unwrap();

        let mut args = SessionRunArgs::new();

        args.add_feed(&states_arg_op, 0, &states_tensor);
        args.add_feed(&next_states_arg_op, 0, &next_states_tensor);
        args.add_feed(&actions_arg_op, 0, &actions_tensor);
        args.add_feed(&rewards_arg_op, 0, &rewards_tensor);
        args.add_feed(&dones_arg_op, 0, &dones_tensor);
        args.add_feed(&probabilities_arg_op, 0, &probabilities_tensor);
        args.add_feed(&min_probability_arg_op, 0, &min_probability_tensor);
        args.add_feed(&replay_memory_len_arg_op, 0, &replay_memory_len_tensor);
        args.add_feed(&beta_arg_op, 0, &beta_tensor);

        let output_loss_fetch_token = args.request_fetch(&output_loss_op, 0);
        let output_abs_td_errors_fetch_token = args.request_fetch(&output_abs_td_errors_op, 1);

        session
            .run(&mut args)
            .expect("train_batch_prioritized couldn't run session");

        let output_loss_tensor : Tensor<f32> = args.fetch(output_loss_fetch_token).unwrap();
        let output_abs_td_errors_tensor : Tensor<f32> = args.fetch(output_abs_td_errors_fetch_token).unwrap();

        (output_loss_tensor.get(&[]), output_abs_td_errors_tensor.iter().map(|x| *x as f64).collect::<Vec<_>>())
    }
    pub fn copy_control_to_target(&mut self) {
        let session = &self.model_bundle.session;
        let signature = self.model_bundle.meta_graph_def().get_signature("copy_control_to_target").unwrap();

        let output_info = signature.get_output("output_0").unwrap();

        let output_op = self.graph.operation_by_name_required(&output_info.name().name).unwrap();

        let mut args = SessionRunArgs::new();

        let output_fetch_token = args.request_fetch(&output_op, 0);

        session
            .run(&mut args)
            .expect("copy_control_to_target couldn't run session");

        let _ : Tensor<i32> = args.fetch(output_fetch_token).unwrap();
    }
    pub fn save(&self, filepath : &str) {
        let session = &self.model_bundle.session;
        let signature = self.model_bundle.meta_graph_def().get_signature("save").unwrap();

        let path_arg_info = signature.get_input("path").unwrap();
        let output_info = signature.get_output("output_0").unwrap();

        let path_arg_op = self.graph.operation_by_name_required(&path_arg_info.name().name).unwrap();
        let output_op = self.graph.operation_by_name_required(&output_info.name().name).unwrap();

        let path_tensor = Tensor::new(&[]).with_values(&[filepath.to_string()]).unwrap();

        let mut args = SessionRunArgs::new();

        args.add_feed(&path_arg_op, 0, &path_tensor);

        let output_fetch_token = args.request_fetch(&output_op, 0);

        session
            .run(&mut args)
            .expect("save couldn't run session");

        let _ : Tensor<i32> = args.fetch(output_fetch_token).unwrap();
    }
    pub fn load(&self, filepath : &str) {
        let session = &self.model_bundle.session;
        let signature = self.model_bundle.meta_graph_def().get_signature("load").unwrap();

        let path_arg_info = signature.get_input("path").unwrap();
        let output_info = signature.get_output("output_0").unwrap();

        let path_arg_op = self.graph.operation_by_name_required(&path_arg_info.name().name).unwrap();
        let output_op = self.graph.operation_by_name_required(&output_info.name().name).unwrap();

        let path_tensor = Tensor::new(&[]).with_values(&[filepath.to_string()]).unwrap();

        let mut args = SessionRunArgs::new();

        args.add_feed(&path_arg_op, 0, &path_tensor);

        let output_fetch_token = args.request_fetch(&output_op, 0);

        session
            .run(&mut args)
            .expect("load couldn't run session");

        let _ : Tensor<i32> = args.fetch(output_fetch_token).unwrap();
    }
}
