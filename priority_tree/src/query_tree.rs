use super::tree::Tree;
use serde::{Deserialize, Serialize};

pub trait Query<V> {
    fn default() -> V;
    fn children_query(tree: &Tree<V>, node: usize) -> V;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QueryTree<V, Q: Query<V>> {
    tree: Tree<V>,
    _marker: std::marker::PhantomData<Q>,
}

impl<V: Copy, Q: Query<V>> QueryTree<V, Q> {
    pub fn with_leaf_count(leaf_count: usize) -> Self {
        Self {
            tree: Tree::new(Q::default(), leaf_count),
            _marker: std::marker::PhantomData,
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

    // assumes a leaf was provided
    pub fn update_value(&mut self, leaf: usize, value: V) {
        self.tree.set_value(leaf, value);
        self.update_ancestors(leaf);
    }

    // assumes a leaf was provided
    pub fn reset_value(&mut self, leaf: usize) {
        self.update_value(leaf, Q::default());
    }

    fn update_ancestors(&mut self, mut node: usize) {
        while let Some(parent) = self.tree.parent(node) {
            let value = Q::children_query(&self.tree, parent);
            self.tree.set_value(parent, value);
            node = parent;
        }
    }
}
