use super::FrameStack;
use super::{State, Transition};
use crate::ImageOwned2;
use std::collections::VecDeque;

pub struct Episode {
    state: FrameStack,
    score: u32,
    truncation_timer: u32,
}

pub enum Status {
    Running,
    Done(Done),
}

pub enum Done {
    // the naming discrepancy between Terminated and ShouldTruncate is
    // intended. it highlights a semantic difference: termination is
    // performed automatically, while truncation is the caller's
    // reponsibility. this may seem strange, but it follows from the
    // fact that termination is triggered by the game (upon death) and
    // truncation is triggered externally (via truncation_timer)
    //
    // in practice, this means that the caller has to reset the episode
    // on truncation, but not on termination
    Terminated,
    ShouldTruncate,
}

impl Episode {
    pub fn new(frame: ImageOwned2, score: u32) -> Self {
        Self {
            state: FrameStack::from(frame),
            score,
            truncation_timer: 0,
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
        if score == next_score {
            self.truncation_timer += 1;
        } else {
            self.truncation_timer = 0;
        }
        if terminated {
            *self = Self::new(next_frame, next_score);
        } else {
            self.state.push(next_frame);
            self.score = next_score;
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
        } else if self.truncation_timer_exceeded_threshold() {
            Status::Done(Done::ShouldTruncate)
        } else {
            Status::Running
        }
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
    fn truncation_timer_exceeded_threshold(&self) -> bool {
        const TIMER_THRESHOLD: u32 = 200;
        self.truncation_timer >= TIMER_THRESHOLD
    }
    pub fn state(&self) -> State {
        // Two clones are needed here. The first casts the &StateStack
        // into a &mut StateStack, because as_slice takes &mut self
        // while this function's signature requires &self. The second
        // clone extracts an owned State from the resulting &State
        self.state.clone().as_slice().clone()
    }
}
