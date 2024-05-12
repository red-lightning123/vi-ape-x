mod max_tree;
mod min_tree;
mod nodes;
mod priority_tree;
mod query_tree;
mod sum_tree;
mod traits;
mod tree;

use max_tree::MaxTree;
use min_tree::MinTree;
use nodes::{MaxNode, MinNode};
use priority_tree::{Priority, PriorityTree};
use rand::distributions::{Distribution, Standard};
use rand::Rng;
use std::ops::Div;
use sum_tree::SumTree;
pub use traits::{Infinity, NegativeInfinity, Zero};

pub struct PriorityCircBuffer<P: Copy + Zero + std::ops::Add<Output = P> + PartialOrd, V> {
    priorities: SumTree<P>,
    priorities_min: MinTree<MinNode<P>>,
    priorities_max: MaxTree<MaxNode<P>>,
    first_priority_leaf: usize,
    values: Vec<V>,
    max_size: usize,
    head: usize,
}

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

impl<P: Priority, V> PriorityCircBuffer<P, V>
where
    Standard: Distribution<<P as Div>::Output>,
    <P as Div>::Output: PartialOrd,
{
    pub fn sample_from_range<R>(&self, range_start: P, range_end: P, rng: &mut R) -> (usize, P, &V)
    where
        R: Rng,
    {
        let index = self
            .priorities
            .sample_from_range(range_start, range_end, rng);
        let value_index = index - self.first_priority_leaf;
        let priority = self.priorities.value(index);
        let value = &self.values[value_index];
        (index, priority, value)
    }
    pub fn sample<R>(&self, rng: &mut R) -> (usize, P, &V)
    where
        R: Rng,
    {
        // TODO: the actual tree node index is an implementation
        // detail so it should be encapsulated in a wrapper type
        let index = self.priorities.sample(rng);
        let value_index = index - self.first_priority_leaf;
        let priority = self.priorities.value(index);
        let value = &self.values[value_index];
        (index, priority, value)
    }
}

use super::transition_saving::{load_transitions, save_transitions};
use super::Transition;
use crate::file_io::{create_file_buf_write, open_file_buf_read};
use std::path::Path;

impl PriorityCircBuffer<f64, Transition> {
    pub fn save<P: AsRef<Path>>(&self, path: P) {
        let path = path.as_ref();
        let max_size_file = create_file_buf_write(path.join("max_size")).unwrap();
        bincode::serialize_into(max_size_file, &self.max_size).unwrap();
        save_transitions(path, &self.values);
        let head_file = create_file_buf_write(path.join("head")).unwrap();
        bincode::serialize_into(head_file, &self.head).unwrap();
        let first_priority_leaf_file =
            create_file_buf_write(path.join("first_priority_leaf")).unwrap();
        bincode::serialize_into(first_priority_leaf_file, &self.first_priority_leaf).unwrap();
        let priority_sum_tree_file = create_file_buf_write(path.join("priority_sum_tree")).unwrap();
        bincode::serialize_into(priority_sum_tree_file, &self.priorities).unwrap();
        let priority_min_tree_file = create_file_buf_write(path.join("priority_min_tree")).unwrap();
        bincode::serialize_into(priority_min_tree_file, &self.priorities_min).unwrap();
        let priority_max_tree_file = create_file_buf_write(path.join("priority_max_tree")).unwrap();
        bincode::serialize_into(priority_max_tree_file, &self.priorities_max).unwrap();
    }
    pub fn load<P: AsRef<Path>>(&mut self, path: P) {
        let path = path.as_ref();
        let max_size_file = open_file_buf_read(path.join("max_size")).unwrap();
        self.max_size = bincode::deserialize_from(max_size_file).unwrap();
        self.values = load_transitions(path, self.max_size);
        let head_file = open_file_buf_read(path.join("head")).unwrap();
        self.head = bincode::deserialize_from(head_file).unwrap();
        let first_priority_leaf_file =
            open_file_buf_read(path.join("first_priority_leaf")).unwrap();
        self.first_priority_leaf = bincode::deserialize_from(first_priority_leaf_file).unwrap();
        let priority_sum_tree_file = open_file_buf_read(path.join("priority_sum_tree")).unwrap();
        self.priorities = bincode::deserialize_from(priority_sum_tree_file).unwrap();
        let priority_min_tree_file = open_file_buf_read(path.join("priority_min_tree")).unwrap();
        self.priorities_min = bincode::deserialize_from(priority_min_tree_file).unwrap();
        let priority_max_tree_file = open_file_buf_read(path.join("priority_max_tree")).unwrap();
        self.priorities_max = bincode::deserialize_from(priority_max_tree_file).unwrap();
    }
}
