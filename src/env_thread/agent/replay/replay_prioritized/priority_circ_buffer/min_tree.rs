use super::{tree::Tree, Infinity};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct MinTree<V> {
    tree: Tree<V>,
}

impl<V: Infinity + Clone + Copy + PartialOrd> MinTree<V> {
    pub fn with_leaf_count(leaf_count: usize) -> Self {
        Self {
            tree: Tree::new(V::infinity(), leaf_count),
        }
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

    fn children_value_min(&self, node: usize) -> V {
        match self.tree.children(node) {
            (None, None) => V::infinity(),
            (Some(left), None) => self.tree.value(left),
            (None, Some(right)) => self.tree.value(right),
            (Some(left), Some(right)) => {
                let left_value = self.tree.value(left);
                let right_value = self.tree.value(right);
                if left_value < right_value {
                    left_value
                } else {
                    right_value
                }
            }
        }
    }

    fn update_ancestors(&mut self, mut node: usize) {
        while let Some(parent) = self.tree.parent(node) {
            let min = self.children_value_min(parent);
            self.tree.set_value(parent, min);
            node = parent;
        }
    }
}
