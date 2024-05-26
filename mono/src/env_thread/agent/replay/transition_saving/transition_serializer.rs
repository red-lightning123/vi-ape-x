use replay_data::{
    CompressedImageOwned2, CompressedRcState, CompressedRcTransition, SavedState, SavedTransition,
};
use std::collections::HashMap;
use std::rc::Rc;

pub struct TransitionSerializer<'a> {
    frames: Vec<&'a Rc<CompressedImageOwned2>>,
    transitions: Vec<SavedTransition>,
    frame_pointers_to_indices: HashMap<*const CompressedImageOwned2, usize>,
}

impl<'a> TransitionSerializer<'a> {
    pub fn new() -> Self {
        Self {
            frames: vec![],
            transitions: vec![],
            frame_pointers_to_indices: HashMap::new(),
        }
    }

    pub fn run<I>(
        mut self,
        transitions: I,
    ) -> (Vec<&'a Rc<CompressedImageOwned2>>, Vec<SavedTransition>)
    where
        I: IntoIterator<Item = &'a CompressedRcTransition>,
    {
        for transition in transitions {
            self.receive_transition(transition);
        }
        (self.frames, self.transitions)
    }

    fn receive_transition(&mut self, transition: &'a CompressedRcTransition) {
        let transition = SavedTransition {
            state: self.receive_state(&transition.state),
            next_state: self.receive_state(&transition.next_state),
            action: transition.action,
            reward: transition.reward,
            terminated: transition.terminated,
        };
        self.transitions.push(transition);
    }

    fn receive_state(&mut self, state: &'a CompressedRcState) -> SavedState {
        let state_frame_indices = state
            .frames()
            .each_ref()
            .map(|frame| self.receive_frame(frame));
        state_frame_indices.into()
    }

    fn receive_frame(&mut self, frame: &'a Rc<CompressedImageOwned2>) -> usize {
        if let Some(frame_index) = self.frame_pointers_to_indices.get(&Rc::as_ptr(frame)) {
            *frame_index
        } else {
            let frame_index = self.frames.len();
            self.frames.push(frame);
            self.frame_pointers_to_indices
                .insert(Rc::as_ptr(frame), frame_index);
            frame_index
        }
    }
}
