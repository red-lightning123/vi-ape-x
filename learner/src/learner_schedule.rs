use file_io::{create_file_buf_write, has_data_left, open_file_buf_read};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Serialize, Deserialize)]
pub struct LearnerSchedule {
    n_step: u32,
    target_update_interval_steps: u32,
}

impl LearnerSchedule {
    pub fn new(target_update_interval_steps: u32) -> Self {
        Self {
            n_step: 0,
            target_update_interval_steps,
        }
    }
    pub fn n_step(&self) -> u32 {
        self.n_step
    }
    pub fn step(&mut self) {
        self.n_step += 1;
    }
    pub fn is_time_to_update_target(&self) -> bool {
        self.n_step % self.target_update_interval_steps == 0
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
