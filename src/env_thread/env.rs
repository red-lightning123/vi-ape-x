use super::{EnvThreadMessage, State, Transition};
use crate::GameThreadMessage;
use crate::ImageOwned2;
use crossbeam_channel::{Receiver, Sender};
use std::collections::VecDeque;
use std::rc::Rc;

pub struct Env {
    state: VecDeque<Rc<ImageOwned2>>,
    score: u32,
    receiver: Receiver<EnvThreadMessage>,
    game_thread_sender: Sender<GameThreadMessage>,
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

pub enum StepError {
    WaitForHoldRequest,
    BadMessage,
}

impl Env {
    pub fn new(
        receiver: Receiver<EnvThreadMessage>,
        game_thread_sender: Sender<GameThreadMessage>,
    ) -> Result<Self, StepError> {
        let mut waiting_hold = false;
        let (frame, score) = Self::raw_wait_for_next_frame(&receiver, &mut waiting_hold)?;
        let frame = Rc::new(frame);
        let state = Self::initial_state_from_frame(frame);
        Ok(Self {
            state,
            score,
            receiver,
            game_thread_sender,
            pending_transitions: VecDeque::new(),
            truncation_timer: 0,
            waiting_hold,
        })
    }
    fn initial_state_from_frame(frame_1: Rc<ImageOwned2>) -> VecDeque<Rc<ImageOwned2>> {
        let frame_2 = Rc::clone(&frame_1);
        let frame_3 = Rc::clone(&frame_1);
        let frame_4 = Rc::clone(&frame_1);
        VecDeque::from([frame_1, frame_2, frame_3, frame_4])
    }
    pub fn step(&mut self, action: u8) -> Result<(), StepError> {
        let state_slice = Self::state_as_slice(&mut self.state).clone();
        let score = self.score;
        self.game_thread_sender
            .send(GameThreadMessage::Action(action))
            .unwrap();
        let (next_frame, next_score) = self.wait_for_next_frame()?;
        let next_frame = Rc::new(next_frame);
        let terminated = Self::is_episode_terminated(score, next_score);

        self.score = next_score;
        if terminated {
            self.state = Self::initial_state_from_frame(next_frame);
        } else {
            self.state.pop_front();
            self.state.push_back(Rc::clone(&next_frame));
        }
        let next_state_slice = Self::state_as_slice(&mut self.state).clone();
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
    fn wait_for_next_frame(&mut self) -> Result<(ImageOwned2, u32), StepError> {
        Self::raw_wait_for_next_frame(&self.receiver, &mut self.waiting_hold)
    }
    fn raw_wait_for_next_frame(
        receiver: &Receiver<EnvThreadMessage>,
        waiting_hold: &mut bool,
    ) -> Result<(ImageOwned2, u32), StepError> {
        loop {
            match receiver.recv().unwrap() {
                EnvThreadMessage::Frame(message) => return Ok(message),
                EnvThreadMessage::Master(_) => return Err(StepError::BadMessage),
                EnvThreadMessage::WaitForHold => {
                    *waiting_hold = true;
                }
            }
        }
    }
    fn truncation_timer_exceeded_threshold(&self) -> bool {
        const TIMER_THRESHOLD: u32 = 200;
        self.truncation_timer >= TIMER_THRESHOLD
    }
    fn truncate(&mut self) -> Result<(), StepError> {
        self.game_thread_sender
            .send(GameThreadMessage::Truncation)
            .unwrap();
        self.next_game()?;
        let (frame, score) = self.wait_for_next_frame()?;
        let frame = Rc::new(frame);
        self.state = Self::initial_state_from_frame(frame);
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
    fn state_as_slice(state: &mut VecDeque<Rc<ImageOwned2>>) -> &State {
        <&State>::try_from(&*state.make_contiguous()).unwrap()
    }
    pub fn state(&self) -> State {
        <&State>::try_from(&*self.state.clone().make_contiguous())
            .unwrap()
            .clone()
    }
    pub fn pop_transition(&mut self) -> Option<(Transition, u32)> {
        self.pending_transitions.pop_front()
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
