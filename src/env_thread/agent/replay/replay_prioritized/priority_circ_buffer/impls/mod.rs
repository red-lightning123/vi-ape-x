mod sampling;
mod save_load;

use super::transition_saving;
use super::{MaxTree, MinTree, SumTree};
use super::{Priority, PriorityTree};
use super::{PriorityCircBuffer, Transition};

impl<P: Priority, V> PriorityCircBuffer<P, V> {
    pub fn with_max_size(max_size: usize) -> Self {
        let priorities = SumTree::with_leaf_count(max_size);
        let first_priority_leaf = priorities.first_leaf();
        Self {
            priorities,
            priorities_min: MinTree::with_leaf_count(max_size),
            priorities_max: MaxTree::with_leaf_count(max_size),
            first_priority_leaf,
            values: vec![],
            max_size,
            head: 0,
        }
    }

    pub fn push(&mut self, priority: P, value: V) {
        let leaf = self.first_priority_leaf + self.head;
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
        self.priorities_min.value(self.priorities_min.root()).into()
    }

    pub fn max_priority(&self) -> Option<P> {
        self.priorities_max.value(self.priorities_max.root()).into()
    }

    pub fn total_priority(&self) -> P {
        self.priorities.value(self.priorities.root())
    }

    pub fn update_priority(&mut self, leaf: usize, priority: P) {
        self.priorities.update_value(leaf, priority);
        self.priorities_min.update_value(leaf, priority.into());
        self.priorities_max.update_value(leaf, priority.into());
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }
}
