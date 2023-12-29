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
}

impl From<ImageOwned2> for FrameStack {
    fn from(frame: ImageOwned2) -> FrameStack {
        let frame = Rc::new(frame);
        FrameStack {
            stack: VecDeque::from([frame.clone(), frame.clone(), frame.clone(), frame]),
        }
    }
}
