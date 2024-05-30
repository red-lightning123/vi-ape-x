use super::EnvThreadMessage;
use crate::GameThreadMessage;
use crossbeam_channel::{Receiver, Sender};
use image::ImageOwned2;

pub enum Request {
    Action(u8),
    Truncation,
}

impl From<Request> for GameThreadMessage {
    fn from(request: Request) -> Self {
        match request {
            Request::Action(action) => Self::Action(action),
            Request::Truncation => Self::Truncation,
        }
    }
}

pub struct Reply {
    pub frame: ImageOwned2,
    pub score: u32,
    pub received_wait_for_hold: bool,
}

pub struct MessageBridge {
    receiver: Receiver<EnvThreadMessage>,
    game_thread_sender: Sender<GameThreadMessage>,
}

pub enum StepError {
    WaitForHoldRequest,
    BadMessage,
}

impl MessageBridge {
    pub fn new(
        receiver: Receiver<EnvThreadMessage>,
        game_thread_sender: Sender<GameThreadMessage>,
    ) -> Result<(Self, Reply), StepError> {
        let bridge = Self {
            receiver,
            game_thread_sender,
        };
        let reply = bridge.wait_for_next_reply()?;
        Ok((bridge, reply))
    }
    pub fn send(&self, request: Request) -> Result<Reply, StepError> {
        self.game_thread_sender.send(request.into()).unwrap();
        let reply = self.wait_for_next_reply()?;
        Ok(reply)
    }
    fn wait_for_next_reply(&self) -> Result<Reply, StepError> {
        let mut received_wait_for_hold = false;
        loop {
            match self.receiver.recv().unwrap() {
                EnvThreadMessage::Frame((frame, score)) => {
                    let reply = Reply {
                        frame,
                        score,
                        received_wait_for_hold,
                    };
                    return Ok(reply);
                }
                EnvThreadMessage::Master(_) => return Err(StepError::BadMessage),
                EnvThreadMessage::WaitForHold => {
                    received_wait_for_hold = true;
                }
            }
        }
    }
    pub const fn n_actions() -> u8 {
        const JUMP_ENABLED: bool = false;
        if JUMP_ENABLED {
            3
        } else {
            2
        }
    }
}
