use serde::{Deserialize, Serialize};

enum TreeDir {
    Left,
    Right,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Tree<V> {
    nodes: Vec<V>,
    leaf_count: usize,
}

impl<V: Clone + Copy> Tree<V> {
    pub fn new(value: V, leaf_count: usize) -> Self {
        let leaf_count_prev_power_of_two = leaf_count.next_power_of_two() / 2;
        let tree_len_prev_power_of_two = 2 * leaf_count_prev_power_of_two - 1;
        Self {
            nodes: vec![value; tree_len_prev_power_of_two + leaf_count],
            leaf_count,
        }
    }

    pub fn parent(&self, node: usize) -> Option<usize> {
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

    pub fn value(&self, node: usize) -> V {
        self.nodes[node]
    }

    pub fn set_value(&mut self, node: usize, value: V) {
        self.nodes[node] = value;
    }
}
