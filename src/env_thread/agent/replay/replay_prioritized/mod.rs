mod priority_circ_buffer;

use super::serialize_transitions;
use crate::env_thread::agent::Transition;
use priority_circ_buffer::{PriorityCircBuffer, Zero};
use std::path::Path;

impl Zero for f64 {
    fn zero() -> Self {
        0.0
    }
}

const EPSILON: f64 = 0.001;

pub struct ReplayPrioritized {
    transitions: PriorityCircBuffer<f64, Transition>,
}

impl ReplayPrioritized {
    fn add_transition_with_priority(&mut self, transition: Transition, priority: f64) {
        self.transitions.push(priority, transition);
    }
    fn initial_priority(&self) -> f64 {
        // technically, here OpenAI baselines use the maximum priority over all
        // transitions ever encountered as the placeholder priority, while this
        // uses the maximum over transitions currently present in memory
        let max_priority = self.transitions.max_priority();
        match max_priority {
            Some(max_priority) => {
                if max_priority < EPSILON {
                    EPSILON
                } else {
                    max_priority
                }
            }
            None => EPSILON,
        }
    }
    pub fn with_max_size(max_size: usize) -> Self {
        Self {
            transitions: PriorityCircBuffer::with_max_size(max_size),
        }
    }
    pub fn update_priorities_with_td_errors(
        &mut self,
        indices: &[usize],
        abs_td_errors: &[f64],
        alpha: f64,
    ) {
        for (index, abs_td_error) in indices.iter().zip(abs_td_errors.iter()) {
            let priority = (abs_td_error + EPSILON).powf(alpha);
            self.transitions.update_priority(*index, priority);
        }
    }
    pub fn add_transition(&mut self, transition: Transition) {
        self.add_transition_with_priority(transition, self.initial_priority());
    }
    pub fn sample_batch(&self, batch_size: usize) -> (Vec<usize>, Vec<f64>, Vec<&Transition>) {
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
        let min_priority = self.transitions.min_priority().unwrap_or(EPSILON);
        min_priority / self.transitions.total_priority()
    }
    pub fn len(&self) -> usize {
        self.transitions.len()
    }
    pub fn save<P: AsRef<Path>>(&self, path: P) {
        self.transitions.save(path);
    }
    pub fn load<P: AsRef<Path>>(&mut self, path: P) {
        self.transitions.load(path);
    }
}
