use super::{State, Transition};

pub trait Model {
    fn best_action(&self, state: &State) -> u8;
    fn train_batch(&mut self, batch: &[&Transition]) -> f32;
    fn train_batch_prioritized(
        &mut self,
        batch_transitions: &[&Transition],
        batch_probabilities: &[f64],
        min_probability: f64,
        replay_memory_len: usize,
        beta: f64,
    ) -> (f32, Vec<f64>);
    fn copy_control_to_target(&mut self);
    fn save(&self, filepath: &str);
    fn load(&self, filepath: &str);
}
