mod frame_stack;
mod step_memory;

use super::{Done, Status};
use frame_stack::FrameStack;
use replay_data::{CompressedImageOwned2, CompressedRcState, CompressedRcTransition};
use std::collections::VecDeque;
use step_memory::StepMemory;

pub struct BasicEpisode {
    step_memory: StepMemory,
    state: FrameStack,
    score: u32,
}

impl BasicEpisode {
    pub fn new(frame: CompressedImageOwned2, score: u32) -> Self {
        const N_STEPS: usize = 3;
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
        next_frame: CompressedImageOwned2,
        next_score: u32,
        transition_queue: &mut VecDeque<(CompressedRcTransition, Option<u32>)>,
    ) -> Status {
        let state = self.state.as_state();
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
        if let Some(transition) = self.step_memory.push(state, score, action, reward) {
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
    pub fn state(&self) -> CompressedRcState {
        self.state.as_state()
    }
    pub fn score(&self) -> u32 {
        self.score
    }
}
