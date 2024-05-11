use super::{tree::Tree, NegativeInfinity};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct MaxTree<V> {
    tree: Tree<V>,
}

impl<V: NegativeInfinity + Clone + Copy + PartialOrd> MaxTree<V> {
    pub fn with_leaf_count(leaf_count: usize) -> Self {
        Self {
            tree: Tree::new(V::negative_infinity(), leaf_count),
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

    fn children_value_max(&self, node: usize) -> V {
        match self.tree.children(node) {
            (None, None) => V::negative_infinity(),
            (Some(left), None) => self.tree.value(left),
            (None, Some(right)) => self.tree.value(right),
            (Some(left), Some(right)) => {
                let left_value = self.tree.value(left);
                let right_value = self.tree.value(right);
                if left_value < right_value {
                    right_value
                } else {
                    left_value
                }
            }
        }
    }

    fn update_ancestors(&mut self, mut node: usize) {
        while let Some(parent) = self.tree.parent(node) {
            let max = self.children_value_max(parent);
            self.tree.set_value(parent, max);
            node = parent;
        }
    }
}
