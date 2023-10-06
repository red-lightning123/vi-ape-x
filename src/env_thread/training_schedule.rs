use crate::file_io::{create_file_buf_write, has_data_left, open_file_buf_read};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Serialize, Deserialize)]
pub struct TrainingSchedule {
    n_step: u32,
    eps_min: f64,
    eps_max: f64,
    n_eps_random_steps: u32,
    n_eps_greedy_steps: u32,
    target_update_interval_steps: u32,
}

impl TrainingSchedule {
    pub fn new(
        eps_min: f64,
        eps_max: f64,
        n_eps_random_steps: u32,
        n_eps_greedy_steps: u32,
        target_update_interval_steps: u32,
    ) -> TrainingSchedule {
        TrainingSchedule {
            n_step: 0,
            eps_min,
            eps_max,
            n_eps_random_steps,
            n_eps_greedy_steps,
            target_update_interval_steps,
        }
    }
    pub fn eps(&self) -> f64 {
        let eps = self.eps_max
            + (self.eps_min - self.eps_max) * f64::from(self.n_step - self.n_eps_random_steps)
                / f64::from(self.n_eps_greedy_steps);
        if eps < self.eps_min {
            self.eps_min
        } else {
            eps
        }
    }
    pub fn is_on_eps_random(&self) -> bool {
        self.n_step < self.n_eps_random_steps
    }
    pub fn n_step(&self) -> u32 {
        self.n_step
    }
    pub fn step(&mut self) {
        self.n_step += 1;
    }
    pub fn is_time_to_update_target(&self) -> bool {
        (self.n_step - self.n_eps_random_steps) % self.target_update_interval_steps == 0
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
