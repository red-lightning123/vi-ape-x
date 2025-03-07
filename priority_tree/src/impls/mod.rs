mod sampling;

use super::MaxTree;
use super::MinTree;
use super::SumTree;
pub use super::{Priority, PriorityTree};

impl<P: Priority> PriorityTree<P> {
    pub fn with_leaf_count(leaf_count: usize) -> Self {
        let sum_tree = SumTree::with_leaf_count(leaf_count);
        let first_leaf = sum_tree.first_leaf();
        Self {
            sum_tree,
            min_tree: MinTree::with_leaf_count(leaf_count),
            max_tree: MaxTree::with_leaf_count(leaf_count),
            first_leaf,
        }
    }

    pub fn min(&self) -> Option<P> {
        self.min_tree.value(self.min_tree.root()).into()
    }

    pub fn max(&self) -> Option<P> {
        self.max_tree.value(self.max_tree.root()).into()
    }

    pub fn total(&self) -> P {
        self.sum_tree.value(self.sum_tree.root())
    }

    pub fn priority(&self, index: usize) -> P {
        let leaf = self.first_leaf + index;
        self.sum_tree.value(leaf)
    }

    pub fn update(&mut self, index: usize, priority: P) {
        let leaf = self.first_leaf + index;
        self.sum_tree.update_value(leaf, priority);
        self.min_tree.update_value(leaf, priority.into());
        self.max_tree.update_value(leaf, priority.into());
    }

    pub fn reset(&mut self, index: usize) {
        let leaf = self.first_leaf + index;
        self.sum_tree.reset_value(leaf);
        self.min_tree.reset_value(leaf);
        self.max_tree.reset_value(leaf);
    }
}
