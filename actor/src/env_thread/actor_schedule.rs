use file_io::{create_file_buf_write, has_data_left, open_file_buf_read};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Serialize, Deserialize)]
pub struct ActorSchedule {
    n_step: u32,
    eps: f64,
    param_update_interval_steps: u32,
}

impl ActorSchedule {
    pub fn new(eps: f64, param_update_interval_steps: u32) -> Self {
        Self {
            n_step: 0,
            eps,
            param_update_interval_steps,
        }
    }
    pub fn eps(&self) -> f64 {
        self.eps
    }
    pub fn n_step(&self) -> u32 {
        self.n_step
    }
    pub fn step(&mut self) {
        self.n_step += 1;
    }
    pub fn is_time_to_update_params(&self) -> bool {
        self.n_step % self.param_update_interval_steps == 0
    }
    pub fn save<P: AsRef<Path>>(&self, path: P) {
        let file = create_file_buf_write(path.as_ref().join("schedule")).unwrap();
        bincode::serialize_into(file, self).unwrap();
    }
    pub fn load<P: AsRef<Path>>(&mut self, path: P) {
        let mut file = open_file_buf_read(path.as_ref().join("schedule")).unwrap();
        *self = bincode::deserialize_from(&mut file).unwrap();
        assert!(
            !has_data_left(file).unwrap(),
            "deserialization of file didn't reach EOF"
        );
    }
}
