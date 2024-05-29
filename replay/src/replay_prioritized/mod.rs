mod priority_circ_buffer;

use priority_circ_buffer::{PriorityCircBuffer, Zero};
use replay_data::CompressedTransition;

impl Zero for f64 {
    fn zero() -> Self {
        0.0
    }
}

pub struct ReplayPrioritized {
    transitions: PriorityCircBuffer<f64, CompressedTransition>,
}

impl ReplayPrioritized {
    pub fn add_transition_with_priority(
        &mut self,
        transition: CompressedTransition,
        priority: f64,
    ) {
        self.transitions.push(priority, transition);
    }
    pub fn with_max_size(max_size: usize) -> Self {
        Self {
            transitions: PriorityCircBuffer::with_max_size(max_size),
        }
    }
    pub fn update_priorities(&mut self, indices: &[usize], priorities: &[f64]) {
        for (index, priority) in indices.iter().zip(priorities.iter()) {
            self.transitions.update_priority(*index, *priority);
        }
    }
    pub fn sample_batch(
        &self,
        batch_size: usize,
    ) -> (Vec<usize>, Vec<f64>, Vec<&CompressedTransition>) {
        let mut batch_indices = vec![];
        let mut batch_probabilities = vec![];
        let mut batch_transitions = vec![];
        let total_priority = self.transitions.total_priority();
        for k in 0..batch_size {
            let range_start = (k as f64) / (batch_size as f64);
            let range_end = range_start + 1.0 / (batch_size as f64);
            let (index, priority, transition) =
                self.transitions
                    .sample_from_range(range_start, range_end, &mut rand::thread_rng());
            let probability = priority / total_priority;
            batch_indices.push(index);
            batch_probabilities.push(probability);
            batch_transitions.push(transition);
        }
        (batch_indices, batch_probabilities, batch_transitions)
    }
    pub fn min_probability(&self) -> f64 {
        const EPSILON: f64 = 0.001;
        let min_priority = self.transitions.min_priority().unwrap_or(EPSILON);
        min_priority / self.transitions.total_priority()
    }
    pub fn len(&self) -> usize {
        self.transitions.len()
    }
}
