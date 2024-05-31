mod episode;
mod frame_stack;
mod message_bridge;

use super::EnvThreadMessage;
use crate::GameThreadMessage;
use crossbeam_channel::{Receiver, Sender};
use episode::{BasicEpisode, Done, Status, TimeLimitedWrapper};
use frame_stack::FrameStack;
pub use message_bridge::StepError;
use message_bridge::{MessageBridge, Reply, Request};
use replay_data::{CompressedImageOwned2, CompressedRcState, CompressedRcTransition};
use std::collections::VecDeque;

pub struct Env {
    bridge: MessageBridge,
    episode: TimeLimitedWrapper<FrameStack>,
    pending_transitions: VecDeque<(CompressedRcTransition, Option<u32>)>,
    waiting_hold: bool,
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
        Ok(Self {
            bridge,
            episode: TimeLimitedWrapper::new(BasicEpisode::new((&frame).into(), score)),
            pending_transitions: VecDeque::new(),
            waiting_hold: received_wait_for_hold,
        })
    }
    pub fn step(&mut self, action: u8) -> Result<(), StepError> {
        let (next_frame, next_score) = self.send(Request::Action(action))?;
        let episode_status = self.episode.step(
            action,
            next_frame,
            next_score,
            &mut self.pending_transitions,
        );
        match episode_status {
            Status::Running => Ok(()),
            Status::Done(done_why) => {
                if self.waiting_hold {
                    Err(StepError::WaitForHoldRequest)
                } else {
                    match done_why {
                        Done::Terminated => Ok(()),
                        Done::ShouldTruncate => self.truncate(),
                    }
                }
            }
        }
    }
    fn truncate(&mut self) -> Result<(), StepError> {
        let (frame, score) = self.send(Request::Truncation)?;
        self.episode = TimeLimitedWrapper::new(BasicEpisode::new(frame, score));
        Ok(())
    }
    fn send(&mut self, request: Request) -> Result<(CompressedImageOwned2, u32), StepError> {
        let Reply {
            frame,
            score,
            received_wait_for_hold,
        } = self.bridge.send(request)?;
        if received_wait_for_hold {
            self.waiting_hold = true;
        }
        Ok(((&frame).into(), score))
    }
    pub fn state(&self) -> CompressedRcState {
        self.episode.state()
    }
    pub fn pop_transition(&mut self) -> Option<(CompressedRcTransition, Option<u32>)> {
        self.pending_transitions.pop_front()
    }
    pub const fn n_actions() -> u8 {
        MessageBridge::n_actions()
    }
}
