use super::State;
use crate::ImageOwned2;
use std::collections::VecDeque;
use std::rc::Rc;

#[derive(Clone)]
pub struct FrameStack {
    stack: VecDeque<Rc<ImageOwned2>>,
}

impl FrameStack {
    pub fn push(&mut self, frame: ImageOwned2) {
        self.stack.pop_front();
        self.stack.push_back(Rc::new(frame));
    }

    pub fn as_state(&self) -> State {
        <[Rc<ImageOwned2>; 4]>::try_from(Vec::from(self.stack.clone()))
            .expect("frame stack len should be 4")
            .into()
    }
    pub fn reset_to_current(&mut self) {
        let frame = self.stack.pop_back().unwrap();
        *self = Self::from(frame);
    }
}

impl From<ImageOwned2> for FrameStack {
    fn from(frame: ImageOwned2) -> Self {
        let frame = Rc::new(frame);
        Self::from(frame)
    }
}

impl From<Rc<ImageOwned2>> for FrameStack {
    fn from(frame: Rc<ImageOwned2>) -> Self {
        Self {
            stack: VecDeque::from([frame.clone(), frame.clone(), frame.clone(), frame]),
        }
    }
}
