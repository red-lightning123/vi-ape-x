mod frame_stack;
use super::{Done, Status};
use super::{State, Transition};
use crate::ImageOwned2;
use frame_stack::FrameStack;
use std::collections::VecDeque;

pub struct BasicEpisode {
    state: FrameStack,
    score: u32,
}

impl BasicEpisode {
    pub fn new(frame: ImageOwned2, score: u32) -> Self {
        Self {
            state: FrameStack::from(frame),
            score,
        }
    }
    pub fn step(
        &mut self,
        action: u8,
        next_frame: ImageOwned2,
        next_score: u32,
        transition_queue: &mut VecDeque<(Transition, u32)>,
    ) -> Status {
        let state_slice = self.state.as_slice().clone();
        let score = self.score;
        let terminated = Self::terminated(score, next_score);
        self.state.push(next_frame);
        self.score = next_score;
        if terminated {
            self.reset_to_current();
        }
        let next_state_slice = self.state.as_slice().clone();
        let reward = if terminated {
            0.0
        } else {
            f64::from(next_score - score)
        };
        transition_queue.push_back((
            (state_slice, next_state_slice, action, reward, terminated),
            score,
        ));
        if terminated {
            Status::Done(Done::Terminated)
        } else {
            Status::Running
        }
    }
    pub fn reset_to_current(&mut self) {
        self.state.reset_to_current();
    }
    fn terminated(score: u32, next_score: u32) -> bool {
        // In theory, the score should only decrease once the game is
        // over, since the player never moves backward. A reasonable
        // termination metric would therefore be score > next_score.
        // However, due to a subtle glitch, collisions with platforms
        // actually can push the agent backward narrowly. Sometimes
        // this leads to fluctuations in the score.
        // So instead we check if next_score is smaller than score by a
        // sensible threshold
        const TERMINATION_SCORE_THRESHOLD: u32 = 10;
        score >= next_score + TERMINATION_SCORE_THRESHOLD
    }
    pub fn state(&self) -> State {
        // Two clones are needed here. The first casts the &StateStack
        // into a &mut StateStack, because as_slice takes &mut self
        // while this function's signature requires &self. The second
        // clone extracts an owned State from the resulting &State
        self.state.clone().as_slice().clone()
    }
    pub fn score(&self) -> u32 {
        self.score
    }
}
