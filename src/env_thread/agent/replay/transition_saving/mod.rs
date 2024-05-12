mod saved_transition;

use crate::env_thread::agent::Transition;
use crate::file_io::{create_file_buf_write, has_data_left, open_file_buf_read};
use crate::ImageOwned2;
use saved_transition::SavedTransition;
use std::collections::HashMap;
use std::path::Path;
use std::rc::Rc;

fn transitions_serialized<'a, I>(values: I) -> (Vec<&'a Rc<ImageOwned2>>, Vec<SavedTransition>)
where
    I: IntoIterator<Item = &'a Transition>,
{
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

pub fn save_transitions<'a, P, I>(path: P, transitions: I)
where
    P: AsRef<Path>,
    I: IntoIterator<Item = &'a Transition>,
{
    let path = path.as_ref();
    let (serialized_frames, serialized_transitions) = transitions_serialized(transitions);

    // the experience replay queue can take up a lot of space, therefore we save each
    // frame/transition separately in a streaming manner so as to not inadvertently clone
    // the entire collection (which would cause a spike in RAM usage and might result in OOM)
    let mut frames_file = create_file_buf_write(path.join("frames")).unwrap();
    for frame in serialized_frames {
        bincode::serialize_into(&mut frames_file, &**frame).unwrap();
    }
    let mut transitions_file = create_file_buf_write(path.join("transitions")).unwrap();
    for transition in serialized_transitions {
        bincode::serialize_into(&mut transitions_file, &transition).unwrap();
    }
}

pub fn load_transitions<P: AsRef<Path>>(path: P, max_size: usize) -> Vec<Transition> {
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
