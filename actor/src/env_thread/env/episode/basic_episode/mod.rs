mod step_memory;

use super::{Done, StateAccum, Status};
use replay_data::GenericTransition;
use std::collections::VecDeque;
use step_memory::StepMemory;

pub struct BasicEpisode<State>
where
    State: StateAccum,
{
    step_memory: StepMemory<<State as StateAccum>::View>,
    state: State,
    score: u32,
}

impl<State> BasicEpisode<State>
where
    State: StateAccum,
    <State as StateAccum>::View: Clone,
{
    pub fn new(frame: State::Frame, score: u32) -> Self {
        const N_STEPS: usize = 3;
        const GAMMA: f64 = 0.99;
        Self {
            step_memory: StepMemory::new(N_STEPS, GAMMA),
            state: State::from_frame(frame),
            score,
        }
    }
    pub fn step(
        &mut self,
        action: u8,
        next_frame: State::Frame,
        next_score: u32,
        transition_queue: &mut VecDeque<(GenericTransition<State::View>, Option<u32>)>,
    ) -> Status {
        let state = self.state.view();
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
        self.state.receive(next_frame);
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
    pub fn state(&self) -> State::View {
        self.state.view()
    }
    pub fn score(&self) -> u32 {
        self.score
    }
}
