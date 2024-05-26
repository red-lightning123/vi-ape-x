mod transition_serializer;

use crate::file_io::{create_file_buf_write, has_data_left, open_file_buf_read};
use replay_data::{CompressedImageOwned2, CompressedRcTransition, SavedTransition};
use std::path::Path;
use std::rc::Rc;
use transition_serializer::TransitionSerializer;

pub fn save_transitions<'a, P, I>(path: P, transitions: I)
where
    P: AsRef<Path>,
    I: IntoIterator<Item = &'a CompressedRcTransition>,
{
    let path = path.as_ref();
    let (serialized_frames, serialized_transitions) = TransitionSerializer::new().run(transitions);

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

pub fn load_transitions<P: AsRef<Path>>(path: P, max_size: usize) -> Vec<CompressedRcTransition> {
    let path = path.as_ref();
    let mut frames_file = open_file_buf_read(path.join("frames")).unwrap();
    let mut frames: Vec<Rc<CompressedImageOwned2>> = vec![];
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
            .state
            .frames()
            .map(|frame_index| Rc::clone(&frames[frame_index]));
        let next_state = saved_transition
            .next_state
            .frames()
            .map(|frame_index| Rc::clone(&frames[frame_index]));
        let transition = CompressedRcTransition {
            state: state.into(),
            next_state: next_state.into(),
            action: saved_transition.action,
            reward: saved_transition.reward,
            terminated: saved_transition.terminated,
        };
        transitions.push(transition);
    }
    transitions
}
