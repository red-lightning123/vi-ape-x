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

    pub fn as_slice(&mut self) -> &State {
        <&State>::try_from(&*self.stack.make_contiguous()).unwrap()
    }
    pub fn reset_to_current(&mut self) {
        let frame = self.stack.pop_back().unwrap();
        self.stack = VecDeque::from([frame.clone(), frame.clone(), frame.clone(), frame]);
    }
}

impl From<ImageOwned2> for FrameStack {
    fn from(frame: ImageOwned2) -> Self {
        let frame = Rc::new(frame);
        Self {
            stack: VecDeque::from([frame.clone(), frame.clone(), frame.clone(), frame]),
        }
    }
}
