mod sum_query;

use super::{
    query_tree::{Query, QueryTree},
    tree::Tree,
    Zero,
};
use serde::{Deserialize, Serialize};
use sum_query::SumQuery;

#[derive(Debug, Serialize, Deserialize)]
pub struct SumTree<V: Copy + Zero + std::ops::Add<Output = V>> {
    tree: QueryTree<V, SumQuery>,
}

impl<V: Copy + Zero + std::ops::Add<Output = V>> SumTree<V> {
    pub fn with_leaf_count(leaf_count: usize) -> Self {
        Self {
            tree: QueryTree::with_leaf_count(leaf_count),
        }
    }

    pub fn children(&self, node: usize) -> (Option<usize>, Option<usize>) {
        self.tree.children(node)
    }

    pub fn root(&self) -> usize {
        self.tree.root()
    }

    pub fn first_leaf(&self) -> usize {
        self.tree.first_leaf()
    }

    pub fn value(&self, node: usize) -> V {
        self.tree.value(node)
    }

    pub fn update_value(&mut self, leaf: usize, value: V) {
        self.tree.update_value(leaf, value)
    }
}
