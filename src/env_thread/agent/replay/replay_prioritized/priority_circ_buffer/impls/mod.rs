mod sampling;
mod save_load;

use super::transition_saving;
use super::{CompressedTransition, PriorityCircBuffer};
use super::{Priority, PriorityTree};

impl<P: Priority, V> PriorityCircBuffer<P, V> {
    pub fn with_max_size(max_size: usize) -> Self {
        Self {
            priorities: PriorityTree::with_leaf_count(max_size),
            values: vec![],
            max_size,
            head: 0,
        }
    }

    pub fn push(&mut self, priority: P, value: V) {
        let leaf = self.priorities.first_leaf() + self.head;
        self.update_priority(leaf, priority);
        if self.head == self.values.len() {
            self.values.push(value);
        } else {
            self.values[self.head] = value;
        }
        self.head += 1;
        if self.head == self.max_size {
            self.head = 0;
        }
    }

    pub fn min_priority(&self) -> Option<P> {
        self.priorities.min()
    }

    pub fn max_priority(&self) -> Option<P> {
        self.priorities.max()
    }

    pub fn total_priority(&self) -> P {
        self.priorities.total()
    }

    pub fn update_priority(&mut self, leaf: usize, priority: P) {
        self.priorities.update(leaf, priority);
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }
}
