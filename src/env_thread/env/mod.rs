mod frame_stack;
use frame_stack::FrameStack;
mod message_bridge;
pub use message_bridge::StepError;
use message_bridge::{MessageBridge, Reply, Request};

use super::{EnvThreadMessage, State, Transition};
use crate::GameThreadMessage;
use crate::ImageOwned2;
use crossbeam_channel::{Receiver, Sender};
use std::collections::VecDeque;

pub struct Env {
    bridge: MessageBridge,
    state: FrameStack,
    score: u32,
    pending_transitions: VecDeque<(Transition, u32)>,
    truncation_timer: u32,
    waiting_hold: bool,
}

impl Env {
    fn is_episode_terminated(score: u32, next_score: u32) -> bool {
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
}

impl Env {
    pub fn new(
        receiver: Receiver<EnvThreadMessage>,
        game_thread_sender: Sender<GameThreadMessage>,
    ) -> Result<Self, StepError> {
        let (bridge, reply) = MessageBridge::new(receiver, game_thread_sender)?;
        let Reply {
            frame,
            score,
            received_wait_for_hold,
        } = reply;
        let state = FrameStack::from(frame);
        Ok(Self {
            bridge,
            state,
            score,
            pending_transitions: VecDeque::new(),
            truncation_timer: 0,
            waiting_hold: received_wait_for_hold,
        })
    }
    pub fn step(&mut self, action: u8) -> Result<(), StepError> {
        let state_slice = self.state.as_slice().clone();
        let score = self.score;
        let (next_frame, next_score) = self.send(Request::Action(action))?;
        let terminated = Self::is_episode_terminated(score, next_score);

        self.score = next_score;
        if terminated {
            self.state = FrameStack::from(next_frame);
        } else {
            self.state.push(next_frame);
        }
        let next_state_slice = self.state.as_slice().clone();
        let reward = if terminated {
            0.0
        } else {
            f64::from(next_score - score)
        };
        self.pending_transitions.push_back((
            (state_slice, next_state_slice, action, reward, terminated),
            score,
        ));
        if terminated {
            self.next_game()?;
        }
        // && !terminated is technically not necessary due to the nature of the game but is here
        // for generality
        if score == next_score && !terminated {
            self.truncation_timer += 1;
        } else {
            self.truncation_timer = 0;
        }
        if self.truncation_timer_exceeded_threshold() {
            self.truncate()?;
        }
        Ok(())
    }
    fn truncation_timer_exceeded_threshold(&self) -> bool {
        const TIMER_THRESHOLD: u32 = 200;
        self.truncation_timer >= TIMER_THRESHOLD
    }
    fn truncate(&mut self) -> Result<(), StepError> {
        self.next_game()?;
        let (frame, score) = self.send(Request::Truncation)?;
        self.state = FrameStack::from(frame);
        self.score = score;
        self.truncation_timer = 0;
        Ok(())
    }
    fn next_game(&self) -> Result<(), StepError> {
        if self.waiting_hold {
            Err(StepError::WaitForHoldRequest)
        } else {
            Ok(())
        }
    }
    fn send(&mut self, request: Request) -> Result<(ImageOwned2, u32), StepError> {
        let Reply {
            frame,
            score,
            received_wait_for_hold,
        } = self.bridge.send(request)?;
        if received_wait_for_hold {
            self.waiting_hold = true;
        }
        Ok((frame, score))
    }
    pub fn state(&self) -> State {
        // Two clones are needed here. The first casts the &StateStack
        // into a &mut StateStack, because as_slice takes &mut self
        // while this function's signature requires &self. The second
        // clone extracts an owned State from the resulting &State
        self.state.clone().as_slice().clone()
    }
    pub fn pop_transition(&mut self) -> Option<(Transition, u32)> {
        self.pending_transitions.pop_front()
    }
    pub const fn n_actions() -> u8 {
        MessageBridge::n_actions()
    }
}
