use super::PriorityTree;
use crate::file_io::{create_file_buf_write, open_file_buf_read};
use std::path::Path;

impl PriorityTree<f64> {
    pub fn save<P: AsRef<Path>>(&self, path: P) {
        let path = path.as_ref();
        let first_leaf_file = create_file_buf_write(path.join("first_leaf")).unwrap();
        bincode::serialize_into(first_leaf_file, &self.first_leaf).unwrap();
        let sum_tree_file = create_file_buf_write(path.join("sum_tree")).unwrap();
        bincode::serialize_into(sum_tree_file, &self.sum_tree).unwrap();
        let min_tree_file = create_file_buf_write(path.join("min_tree")).unwrap();
        bincode::serialize_into(min_tree_file, &self.min_tree).unwrap();
        let max_tree_file = create_file_buf_write(path.join("max_tree")).unwrap();
        bincode::serialize_into(max_tree_file, &self.max_tree).unwrap();
    }
    pub fn load<P: AsRef<Path>>(&mut self, path: P) {
        let path = path.as_ref();
        let first_leaf_file = open_file_buf_read(path.join("first_leaf")).unwrap();
        self.first_leaf = bincode::deserialize_from(first_leaf_file).unwrap();
        let sum_tree_file = open_file_buf_read(path.join("sum_tree")).unwrap();
        self.sum_tree = bincode::deserialize_from(sum_tree_file).unwrap();
        let min_tree_file = open_file_buf_read(path.join("min_tree")).unwrap();
        self.min_tree = bincode::deserialize_from(min_tree_file).unwrap();
        let max_tree_file = open_file_buf_read(path.join("max_tree")).unwrap();
        self.max_tree = bincode::deserialize_from(max_tree_file).unwrap();
    }
}
