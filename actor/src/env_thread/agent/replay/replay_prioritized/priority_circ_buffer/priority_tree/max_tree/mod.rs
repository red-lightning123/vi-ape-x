mod max_query;

use super::{
    query_tree::{Query, QueryTree},
    tree::Tree,
    NegativeInfinity,
};
use max_query::MaxQuery;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct MaxTree<V: Copy + NegativeInfinity + PartialOrd> {
    tree: QueryTree<V, MaxQuery>,
}

impl<V: Copy + NegativeInfinity + PartialOrd> MaxTree<V> {
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
