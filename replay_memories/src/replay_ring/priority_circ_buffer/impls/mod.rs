mod sampling;

use super::PriorityCircBuffer;
use priority_tree::{Priority, PriorityTree};

impl<P: Priority, V> PriorityCircBuffer<P, V> {
    pub fn with_max_size(max_size: usize) -> Self {
        Self {
            priorities: PriorityTree::with_leaf_count(max_size),
            values: Self::vec_of_nones(max_size),
            max_size,
            head: 0,
            tail: 0,
        }
    }

    fn vec_of_nones<T>(len: usize) -> Vec<Option<T>> {
        std::iter::repeat_with(|| None).take(len).collect()
    }

    pub fn truncate(&mut self, truncated_len: usize) {
        if self.len() > truncated_len {
            let new_tail = self.mod_max_size(self.head as isize - truncated_len as isize);
            while self.tail != new_tail {
                self.reset_entry(self.tail);
                self.tail += 1;
                if self.tail == self.max_size {
                    self.tail = 0;
                }
            }
        }
    }

    fn reset_entry(&mut self, index: usize) {
        self.priorities.reset(index);
        self.values[index] = None;
    }

    pub fn push(&mut self, priority: P, value: V) {
        self.update_priority(self.head, priority);
        self.values[self.head] = Some(value);
        self.head += 1;
        if self.head == self.max_size {
            self.head = 0;
        }
        assert_ne!(
            self.head, self.tail,
            "must not push into PriorityCircBuffer if it is already full"
        )
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

    pub fn update_priority(&mut self, index: usize, priority: P) {
        self.priorities.update(index, priority);
    }

    pub fn len(&self) -> usize {
        self.mod_max_size(self.head as isize - self.tail as isize)
    }

    fn mod_max_size(&self, n: isize) -> usize {
        n.rem_euclid(self.max_size as isize) as usize
    }
}
