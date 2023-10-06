use crate::env_thread::Transition;
use std::path::Path;

pub trait ReplayMemory {
    fn with_max_size(max_size: usize) -> Self;
    fn add_transition(&mut self, transition: Transition);
    fn sample_batch(&self, _batch_size: usize) -> Vec<&Transition> {
        unimplemented!()
    }
    fn len(&self) -> usize;
    fn save<P: AsRef<Path>>(&self, path: P);
    fn load<P: AsRef<Path>>(&mut self, path: P);

    fn update_priorities_with_td_errors(
        &mut self,
        _indices: &[usize],
        _abs_td_errors: &[f64],
        _alpha: f64,
    ) {
        unimplemented!()
    }
    fn sample_batch_prioritized(
        &self,
        _batch_size: usize,
    ) -> (Vec<usize>, Vec<f64>, Vec<&Transition>) {
        unimplemented!()
    }
    fn min_probability(&self) -> f64 {
        unimplemented!()
    }
}
