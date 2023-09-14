use super::{ State, Transition };
use crate::ImageOwned;
use std::fs;
use std::path::Path;
use tensorflow::{ Graph, SavedModelBundle, SessionOptions, SessionRunArgs, Tensor };
pub mod replay;
use replay::ReplayMemory;

fn state_to_pixels(state : &State) -> Vec<u8> {
    [(*state[0]).as_ref().data(), (*state[1]).as_ref().data(), (*state[2]).as_ref().data(), (*state[3]).as_ref().data()].concat()
}

struct Model {
    model_bundle : SavedModelBundle,
    graph : Graph
}

impl Model {
    fn new() -> Model {
        let mut graph = Graph::new();
        let model_bundle = SavedModelBundle::load(&SessionOptions::new(), ["serve"], &mut graph, "model").expect("Couldn't load model");
        Model {
            model_bundle,
            graph
        }
    }
    fn best_action(&self, state : &State) -> u8 {
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
    fn train_batch(&mut self, batch : &[&Transition]) -> f32 {
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
    fn copy_control_to_target(&mut self) {
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
    fn save(&self, filepath : &str) {
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
    fn load(&self, filepath : &str) {
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

pub struct Agent<R : ReplayMemory> {
    model : Model,
    memory: R
}

type SavedTransition = ([usize; 4], [usize; 4], u8, f64, bool);

impl<R : ReplayMemory> Agent<R> {
    pub fn with_memory_capacity(memory_capacity : usize) -> Agent<R> {
        Agent {
            model : Model::new(),
            memory: R::with_max_size(memory_capacity)
        }
    }
    pub fn best_action(&self, state : &State) -> u8 {
        self.model.best_action(state)
    }
    pub fn remember(&mut self, transition : Transition) {
        self.memory.add_transition(transition);
    }
    pub fn train_step(&mut self) -> Option<f32> {
        const BATCH_SIZE : usize = 32;
        if self.memory.len() >= BATCH_SIZE {
            let batch = self.memory.sample_batch(BATCH_SIZE);
            Some(self.model.train_batch(&batch))
        } else {
            None
        }
    }
    pub fn copy_control_to_target(&mut self) {
        self.model.copy_control_to_target();
    }
    pub fn save<P : AsRef<Path>>(&self, path : P) {
        let path = path.as_ref();
        self.model.save(path.join("model_vars").to_str().unwrap());
        let memory_path = path.join("memory");
        fs::create_dir_all(&memory_path).unwrap();
        self.memory.save(memory_path);
    }
    pub fn load<P : AsRef<Path>>(&mut self, path : P) {
        let path = path.as_ref();
        self.model.load(path.join("model_vars").to_str().unwrap());
        self.memory.load(path.join("memory"));
    }
}
