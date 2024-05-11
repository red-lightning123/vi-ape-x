use super::{tree::Tree, Zero};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SumTree<V> {
    tree: Tree<V>,
}

impl<V: Zero + Clone + Copy + std::ops::Add<Output = V>> SumTree<V> {
    pub fn with_leaf_count(leaf_count: usize) -> Self {
        Self {
            tree: Tree::new(V::zero(), leaf_count),
        }
    }

    pub fn first_leaf(&self) -> usize {
        self.tree.first_leaf()
    }

    pub fn children(&self, node: usize) -> (Option<usize>, Option<usize>) {
        self.tree.children(node)
    }

    pub fn root(&self) -> usize {
        self.tree.root()
    }

    pub fn value(&self, node: usize) -> V {
        self.tree.value(node)
    }

    // assumes a leaf was provided
    pub fn update_value(&mut self, leaf: usize, value: V) {
        self.tree.set_value(leaf, value);
        self.update_ancestors(leaf);
    }

    fn children_value_sum(&self, node: usize) -> V {
        match self.tree.children(node) {
            (None, None) => V::zero(),
            (Some(left), None) => self.tree.value(left),
            (None, Some(right)) => self.tree.value(right),
            (Some(left), Some(right)) => self.tree.value(left) + self.tree.value(right),
        }
    }

    fn update_ancestors(&mut self, mut node: usize) {
        while let Some(parent) = self.tree.parent(node) {
            let sum = self.children_value_sum(parent);
            self.tree.set_value(parent, sum);
            node = parent;
        }
    }
}
