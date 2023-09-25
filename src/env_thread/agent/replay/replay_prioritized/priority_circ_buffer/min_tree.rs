use serde::{ Serialize, Deserialize };
use super::Infinity;

enum TreeDir {
    Left,
    Right
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MinTree<V> {
    nodes : Vec<V>,
    leaf_count : usize
}

impl<V : Infinity + Clone + Copy + PartialOrd> MinTree<V> {
    pub fn with_leaf_count(leaf_count : usize) -> MinTree<V> {
        let leaf_count_prev_power_of_two = leaf_count.next_power_of_two() / 2;
        let tree_len_prev_power_of_two = 2 * leaf_count_prev_power_of_two - 1;
        MinTree {
            nodes : vec![V::infinity(); tree_len_prev_power_of_two + leaf_count],
            leaf_count
        }
    }

    fn parent(&self, node : usize) -> Option<usize> {
        if node == self.root() {
            None
        } else {
            Some((node - 1) / 2)
        }
    }

    fn child(&self, node : usize, dir : TreeDir) -> Option<usize> {
        let child = match dir {
            TreeDir::Left => 2 * node + 1,
            TreeDir::Right => 2 * node + 2
        };
        if child < self.nodes.len() {
            Some(child)
        } else {
            None
        }
    }

    pub fn children(&self, node : usize) -> (Option<usize>, Option<usize>) {
        (self.child(node, TreeDir::Left), self.child(node, TreeDir::Right))
    }

    // assumes self isn't empty
    pub fn root(&self) -> usize {
        0
    }

    pub fn first_leaf(&self) -> usize {
        let mut node = self.root();
        loop {
            match self.child(node, TreeDir::Left) {
                Some(child) => { node = child; }
                None => break node
            }
        }
    }

    // assumes a leaf was provided
    pub fn next_leaf(&self, mut leaf : usize) -> usize {
        leaf += 1;
        if leaf == self.nodes.len() {
            leaf -= self.leaf_count;
        }
        leaf
    }

    pub fn value(&self, node : usize) -> V {
        self.nodes[node]
    }

    fn set_value(&mut self, node : usize, value : V) {
        self.nodes[node] = value;
    }

    // assumes a leaf was provided
    pub fn update_value(&mut self, leaf : usize, value : V) {
        self.set_value(leaf, value);
        self.update_ancestors(leaf);
    }

    fn children_value_min(&self, node : usize) -> V {
        match self.children(node) {
            (None, None) => V::infinity(),
            (Some(left), None) => self.value(left),
            (None, Some(right)) => self.value(right),
            (Some(left), Some(right)) => {
                let left_value = self.value(left);
                let right_value = self.value(right);
                if left_value < right_value {
                    left_value
                } else {
                    right_value
                }
            }
        }
    }

    fn update_ancestors(&mut self, mut node : usize) {
        while let Some(parent) = self.parent(node) {
            let min = self.children_value_min(parent);
            self.set_value(parent, min);
            node = parent;
        }
    }
}
