mod min_query;

use super::{
    query_tree::{Query, QueryTree},
    tree::Tree,
    Infinity,
};
use min_query::MinQuery;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct MinTree<V: Copy + Infinity + PartialOrd> {
    tree: QueryTree<V, MinQuery>,
}

impl<V: Copy + Infinity + PartialOrd> MinTree<V> {
    pub fn with_leaf_count(leaf_count: usize) -> Self {
        Self {
            tree: QueryTree::with_leaf_count(leaf_count),
        }
    }

    pub fn root(&self) -> usize {
        self.tree.root()
    }

    pub fn value(&self, node: usize) -> V {
        self.tree.value(node)
    }

    pub fn update_value(&mut self, leaf: usize, value: V) {
        self.tree.update_value(leaf, value)
    }
}
