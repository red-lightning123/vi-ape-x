mod frame_stack;
mod step_memory;

use super::{Done, Status};
use super::{State, Transition};
use crate::ImageOwned2;
use frame_stack::FrameStack;
use std::collections::VecDeque;
use step_memory::StepMemory;

pub struct BasicEpisode {
    step_memory: StepMemory,
    state: FrameStack,
    score: u32,
}

impl BasicEpisode {
    pub fn new(frame: ImageOwned2, score: u32) -> Self {
        const N_STEPS: usize = 1;
        const GAMMA: f64 = 0.99;
        Self {
            step_memory: StepMemory::new(N_STEPS, GAMMA),
            state: FrameStack::from(frame),
            score,
        }
    }
    pub fn step(
        &mut self,
        action: u8,
        next_frame: ImageOwned2,
        next_score: u32,
        transition_queue: &mut VecDeque<(Transition, Option<u32>)>,
    ) -> Status {
        let state_slice = self.state.as_slice().clone();
        let score = self.score;
        let terminated = Self::terminated(score, next_score);
        let reward = if terminated {
            0.0
        } else {
            // IMPORTANT: The difference can be negative, so order of operations
            // is critical here. The difference could overflow if it were
            // calculated as f64::from(next_score - score)
            f64::from(next_score) - f64::from(score)
        };
        self.state.push(next_frame);
        self.score = next_score;
        if let Some(transition) = self.step_memory.push(state_slice, score, action, reward) {
            transition_queue.push_back(transition);
        }
        if terminated {
            self.step_memory
                .pop_terminated_transitions_into(transition_queue);
            self.reset_to_current();
            Status::Done(Done::Terminated)
        } else {
            Status::Running
        }
    }
    fn reset_to_current(&mut self) {
        self.step_memory.reset_to_current();
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
