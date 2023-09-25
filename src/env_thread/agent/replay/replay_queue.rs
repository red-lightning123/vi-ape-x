use crate::env_thread::agent::Transition;
use super::SavedTransition;
use crate::ImageOwned2;
use super::ReplayMemory;
use crate::file_io::{ create_file_buf_write, open_file_buf_read, has_data_left };
use std::path::Path;
use std::collections::{ VecDeque, HashMap };
use rand::prelude::{ SliceRandom, IteratorRandom };
use std::rc::Rc;

pub struct ReplayQueue {
    transitions : VecDeque<Transition>,
    max_size : usize
}

impl ReplayMemory for ReplayQueue {
    fn with_max_size(max_size : usize) -> ReplayQueue {
        ReplayQueue {
            transitions: VecDeque::with_capacity(max_size),
            max_size
        }
    }
    fn add_transition(&mut self, transition : Transition) {
        if self.transitions.len() >= self.max_size {
            self.transitions.pop_front();
        }
        self.transitions.push_back(transition);
    }
    fn sample_batch(&self, batch_size : usize) -> Vec<&Transition> {
        let mut batch = self.transitions.iter().choose_multiple(&mut rand::thread_rng(), batch_size);
        batch.shuffle(&mut rand::thread_rng());
        batch
    }
    fn len(&self) -> usize {
        self.transitions.len()
    }
    fn save<P : AsRef<Path>>(&self, path : P) {
        let mut frames = vec![];
        let mut transitions : Vec<SavedTransition> = vec![];
        let mut frame_pointers_to_indices = HashMap::new();
        let mut current_frame_index = 0;
        for (state, next_state, action, reward, terminated) in &self.transitions {
            let mut state_frame_indices = vec![];
            for frame in state {
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
            for frame in next_state {
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
            transitions.push(((*state_frame_indices).try_into().unwrap(), (*next_state_frame_indices).try_into().unwrap(), *action, *reward, *terminated));
        }
        // the experience replay queue can take up a lot of space, therefore we serialize each
        // frame/transition separately in a streaming manner so as to not inadvertently clone
        // the entire queue (which would cause a spike in RAM usage and might result in OOM)
        let max_size_file = create_file_buf_write(path.as_ref().join("max_size")).unwrap();
        bincode::serialize_into(max_size_file, &self.max_size).unwrap();
        let mut frames_file = create_file_buf_write(path.as_ref().join("frames")).unwrap();
        for frame in frames {
            bincode::serialize_into(&mut frames_file, &**frame).unwrap();
        }
        let mut transitions_file = create_file_buf_write(path.as_ref().join("transitions")).unwrap();
        for transition in transitions {
            bincode::serialize_into(&mut transitions_file, &transition).unwrap();
        }
    }
    fn load<P : AsRef<Path>>(&mut self, path : P) {
        let max_size_file = open_file_buf_read(path.as_ref().join("max_size")).unwrap();
        self.max_size = bincode::deserialize_from(max_size_file).unwrap();
        let mut frames_file = open_file_buf_read(path.as_ref().join("frames")).unwrap();
        let mut frames : Vec<Rc<ImageOwned2>> = vec![];
        while has_data_left(&mut frames_file).unwrap() {
            let frame = bincode::deserialize_from(&mut frames_file).unwrap();
            let frame = Rc::new(frame);
            frames.push(frame);
        }
        let mut transitions = VecDeque::with_capacity(self.max_size);
        let mut transitions_file = open_file_buf_read(path.as_ref().join("transitions")).unwrap();
        while has_data_left(&mut transitions_file).unwrap() {
            let (state_frame_indices, next_state_frame_indices, action, reward, terminated) : SavedTransition = bincode::deserialize_from(&mut transitions_file).unwrap();
            let state = state_frame_indices.map(|frame_index| Rc::clone(&frames[frame_index]));
            let next_state = next_state_frame_indices.map(|frame_index| Rc::clone(&frames[frame_index]));
            transitions.push_back((state, next_state, action, reward, terminated));
        }
        self.transitions = transitions;
    }
}
