use super::Zero;
use serde::{Deserialize, Serialize};

enum TreeDir {
    Left,
    Right,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SumTree<V> {
    nodes: Vec<V>,
    leaf_count: usize,
}

impl<V: Zero + Clone + Copy + std::ops::Add<Output = V>> SumTree<V> {
    pub fn with_leaf_count(leaf_count: usize) -> Self {
        let leaf_count_prev_power_of_two = leaf_count.next_power_of_two() / 2;
        let tree_len_prev_power_of_two = 2 * leaf_count_prev_power_of_two - 1;
        Self {
            nodes: vec![V::zero(); tree_len_prev_power_of_two + leaf_count],
            leaf_count,
        }
    }

    fn parent(&self, node: usize) -> Option<usize> {
        if node == self.root() {
            None
        } else {
            Some((node - 1) / 2)
        }
    }

    fn child(&self, node: usize, dir: TreeDir) -> Option<usize> {
        let child = match dir {
            TreeDir::Left => 2 * node + 1,
            TreeDir::Right => 2 * node + 2,
        };
        if child < self.nodes.len() {
            Some(child)
        } else {
            None
        }
    }

    pub fn children(&self, node: usize) -> (Option<usize>, Option<usize>) {
        (
            self.child(node, TreeDir::Left),
            self.child(node, TreeDir::Right),
        )
    }

    // assumes self isn't empty
    pub fn root(&self) -> usize {
        0
    }

    pub fn first_leaf(&self) -> usize {
        let mut node = self.root();
        loop {
            match self.child(node, TreeDir::Left) {
                Some(child) => {
                    node = child;
                }
                None => break node,
            }
        }
    }

    // assumes a leaf was provided
    pub fn next_leaf(&self, mut leaf: usize) -> usize {
        leaf += 1;
        if leaf == self.nodes.len() {
            leaf -= self.leaf_count;
        }
        leaf
    }

    pub fn value(&self, node: usize) -> V {
        self.nodes[node]
    }

    fn set_value(&mut self, node: usize, value: V) {
        self.nodes[node] = value;
    }

    // assumes a leaf was provided
    pub fn update_value(&mut self, leaf: usize, value: V) {
        self.set_value(leaf, value);
        self.update_ancestors(leaf);
    }

    fn children_value_sum(&self, node: usize) -> V {
        match self.children(node) {
            (None, None) => V::zero(),
            (Some(left), None) => self.value(left),
            (None, Some(right)) => self.value(right),
            (Some(left), Some(right)) => self.value(left) + self.value(right),
        }
    }

    fn update_ancestors(&mut self, mut node: usize) {
        while let Some(parent) = self.parent(node) {
            let sum = self.children_value_sum(parent);
            self.set_value(parent, sum);
            node = parent;
        }
    }
}
