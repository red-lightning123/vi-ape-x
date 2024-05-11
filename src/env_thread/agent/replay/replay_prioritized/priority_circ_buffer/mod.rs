mod traits;
pub use traits::{Infinity, NegativeInfinity, Zero};
mod nodes;
use nodes::{MaxNode, MinNode};
mod query_tree;
mod sum_tree;
mod tree;
use sum_tree::SumTree;
mod max_tree;
use max_tree::MaxTree;
mod min_tree;
use min_tree::MinTree;
mod priority_tree;
use priority_tree::{Priority, PriorityTree};
use rand::distributions::{Distribution, Standard};
use rand::Rng;
use std::ops::Div;

pub struct PriorityCircBuffer<P: Copy + Zero + std::ops::Add<Output = P> + PartialOrd, V> {
    priorities: SumTree<P>,
    priorities_min: MinTree<MinNode<P>>,
    priorities_max: MaxTree<MaxNode<P>>,
    first_priority_leaf: usize,
    values: Vec<V>,
    max_size: usize,
    head: usize,
}

impl<P: Priority, V> PriorityCircBuffer<P, V> {
    pub fn with_max_size(max_size: usize) -> Self {
        let priorities = SumTree::with_leaf_count(max_size);
        let first_priority_leaf = priorities.first_leaf();
        Self {
            priorities,
            priorities_min: MinTree::with_leaf_count(max_size),
            priorities_max: MaxTree::with_leaf_count(max_size),
            first_priority_leaf,
            values: vec![],
            max_size,
            head: 0,
        }
    }

    pub fn push(&mut self, priority: P, value: V) {
        self.priorities
            .update_value(self.first_priority_leaf + self.head, priority);
        if self.head == self.values.len() {
            self.values.push(value);
        } else {
            self.values[self.head] = value;
        }
        self.head += 1;
        if self.head == self.max_size {
            self.head = 0;
        }
    }

    pub fn min_priority(&self) -> Option<P> {
        self.priorities_min.value(self.priorities_min.root()).into()
    }

    pub fn max_priority(&self) -> Option<P> {
        self.priorities_max.value(self.priorities_max.root()).into()
    }

    pub fn total_priority(&self) -> P {
        self.priorities.value(self.priorities.root())
    }

    pub fn update_priority(&mut self, leaf: usize, priority: P) {
        self.priorities.update_value(leaf, priority);
        self.priorities_min.update_value(leaf, priority.into());
        self.priorities_max.update_value(leaf, priority.into());
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }
}

impl<P: Priority, V> PriorityCircBuffer<P, V>
where
    Standard: Distribution<<P as Div>::Output>,
    <P as Div>::Output: PartialOrd,
{
    pub fn sample_from_range<R>(&self, range_start: P, range_end: P, rng: &mut R) -> (usize, P, &V)
    where
        R: Rng,
    {
        let index = self
            .priorities
            .sample_from_range(range_start, range_end, rng);
        let value_index = index - self.first_priority_leaf;
        let priority = self.priorities.value(index);
        let value = &self.values[value_index];
        (index, priority, value)
    }
    pub fn sample<R>(&self, rng: &mut R) -> (usize, P, &V)
    where
        R: Rng,
    {
        // TODO: the actual tree node index is an implementation
        // detail so it should be encapsulated in a wrapper type
        let index = self.priorities.sample(rng);
        let value_index = index - self.first_priority_leaf;
        let priority = self.priorities.value(index);
        let value = &self.values[value_index];
        (index, priority, value)
    }
}

use super::{SavedTransition, Transition};
use crate::file_io::{create_file_buf_write, has_data_left, open_file_buf_read};
use crate::ImageOwned2;
use std::collections::HashMap;
use std::path::Path;
use std::rc::Rc;

fn frames_transitions_serialized(
    values: &[Transition],
) -> (Vec<&Rc<ImageOwned2>>, Vec<SavedTransition>) {
    let mut frames = vec![];
    let mut transitions: Vec<SavedTransition> = vec![];
    let mut frame_pointers_to_indices = HashMap::new();
    let mut current_frame_index = 0;
    for transition in values {
        let mut state_frame_indices = vec![];
        for frame in &transition.state {
            let frame_index =
                if let Some(frame_index) = frame_pointers_to_indices.get(&Rc::as_ptr(frame)) {
                    *frame_index
                } else {
                    frames.push(frame);
                    frame_pointers_to_indices.insert(Rc::as_ptr(frame), current_frame_index);
                    let frame_index = current_frame_index;
                    current_frame_index += 1;
                    frame_index
                };
            state_frame_indices.push(frame_index);
        }
        let mut next_state_frame_indices = vec![];
        for frame in &transition.next_state {
            let frame_index =
                if let Some(frame_index) = frame_pointers_to_indices.get(&Rc::as_ptr(frame)) {
                    *frame_index
                } else {
                    frames.push(frame);
                    frame_pointers_to_indices.insert(Rc::as_ptr(frame), current_frame_index);
                    let frame_index = current_frame_index;
                    current_frame_index += 1;
                    frame_index
                };
            next_state_frame_indices.push(frame_index);
        }
        let transition = SavedTransition {
            state_frame_indices: (*state_frame_indices).try_into().unwrap(),
            next_state_frame_indices: (*next_state_frame_indices).try_into().unwrap(),
            action: transition.action,
            reward: transition.reward,
            terminated: transition.terminated,
        };
        transitions.push(transition);
    }
    (frames, transitions)
}

fn values_deserialized<P: AsRef<Path>>(path: P, max_size: usize) -> Vec<Transition> {
    let path = path.as_ref();
    let mut frames_file = open_file_buf_read(path.join("frames")).unwrap();
    let mut frames: Vec<Rc<ImageOwned2>> = vec![];
    while has_data_left(&mut frames_file).unwrap() {
        let frame = bincode::deserialize_from(&mut frames_file).unwrap();
        let frame = Rc::new(frame);
        frames.push(frame);
    }
    let mut transitions = Vec::with_capacity(max_size);
    let mut transitions_file = open_file_buf_read(path.join("transitions")).unwrap();
    while has_data_left(&mut transitions_file).unwrap() {
        let saved_transition: SavedTransition =
            bincode::deserialize_from(&mut transitions_file).unwrap();
        let state = saved_transition
            .state_frame_indices
            .map(|frame_index| Rc::clone(&frames[frame_index]));
        let next_state = saved_transition
            .next_state_frame_indices
            .map(|frame_index| Rc::clone(&frames[frame_index]));
        let transition = Transition {
            state,
            next_state,
            action: saved_transition.action,
            reward: saved_transition.reward,
            terminated: saved_transition.terminated,
        };
        transitions.push(transition);
    }
    transitions
}

impl PriorityCircBuffer<f64, Transition> {
    pub fn save<P: AsRef<Path>>(&self, path: P) {
        // the experience replay queue can take up a lot of space, therefore we serialize each
        // frame/transition separately in a streaming manner so as to not inadvertently clone
        // the entire queue (which would cause a spike in RAM usage and might result in OOM)
        let max_size_file = create_file_buf_write(path.as_ref().join("max_size")).unwrap();
        bincode::serialize_into(max_size_file, &self.max_size).unwrap();
        let (serialized_frames, serialized_transitions) =
            frames_transitions_serialized(&self.values);
        let mut frames_file = create_file_buf_write(path.as_ref().join("frames")).unwrap();
        for frame in serialized_frames {
            bincode::serialize_into(&mut frames_file, &**frame).unwrap();
        }
        let mut transitions_file =
            create_file_buf_write(path.as_ref().join("transitions")).unwrap();
        for transition in serialized_transitions {
            bincode::serialize_into(&mut transitions_file, &transition).unwrap();
        }
        let head_file = create_file_buf_write(path.as_ref().join("head")).unwrap();
        bincode::serialize_into(head_file, &self.head).unwrap();
        let first_priority_leaf_file =
            create_file_buf_write(path.as_ref().join("first_priority_leaf")).unwrap();
        bincode::serialize_into(first_priority_leaf_file, &self.first_priority_leaf).unwrap();
        let priority_sum_tree_file =
            create_file_buf_write(path.as_ref().join("priority_sum_tree")).unwrap();
        bincode::serialize_into(priority_sum_tree_file, &self.priorities).unwrap();
        let priority_min_tree_file =
            create_file_buf_write(path.as_ref().join("priority_min_tree")).unwrap();
        bincode::serialize_into(priority_min_tree_file, &self.priorities_min).unwrap();
        let priority_max_tree_file =
            create_file_buf_write(path.as_ref().join("priority_max_tree")).unwrap();
        bincode::serialize_into(priority_max_tree_file, &self.priorities_max).unwrap();
    }
    pub fn load<P: AsRef<Path>>(&mut self, path: P) {
        let max_size_file = open_file_buf_read(path.as_ref().join("max_size")).unwrap();
        self.max_size = bincode::deserialize_from(max_size_file).unwrap();
        let deserialized_values = values_deserialized(path.as_ref(), self.max_size);
        self.values = deserialized_values;
        let head_file = open_file_buf_read(path.as_ref().join("head")).unwrap();
        self.head = bincode::deserialize_from(head_file).unwrap();
        let first_priority_leaf_file =
            open_file_buf_read(path.as_ref().join("first_priority_leaf")).unwrap();
        self.first_priority_leaf = bincode::deserialize_from(first_priority_leaf_file).unwrap();
        let priority_sum_tree_file =
            open_file_buf_read(path.as_ref().join("priority_sum_tree")).unwrap();
        self.priorities = bincode::deserialize_from(priority_sum_tree_file).unwrap();
        let priority_min_tree_file =
            open_file_buf_read(path.as_ref().join("priority_min_tree")).unwrap();
        self.priorities_min = bincode::deserialize_from(priority_min_tree_file).unwrap();
        let priority_max_tree_file =
            open_file_buf_read(path.as_ref().join("priority_max_tree")).unwrap();
        self.priorities_max = bincode::deserialize_from(priority_max_tree_file).unwrap();
    }
}
