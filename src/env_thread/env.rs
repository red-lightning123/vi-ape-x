use super::{EnvThreadMessage, State, Transition};
use crate::ImageOwned2;
use crate::{GameThreadMessage, PlotThreadMessage};
use crossbeam_channel::{Receiver, Sender};
use std::collections::VecDeque;
use std::rc::Rc;

pub struct Env {
    state: VecDeque<Rc<ImageOwned2>>,
    score: u32,
    receiver: Receiver<EnvThreadMessage>,
    game_thread_sender: Sender<GameThreadMessage>,
    plot_thread_sender: Sender<PlotThreadMessage>,
    pending_transitions: VecDeque<Transition>,
    truncation_timer: u32,
    waiting_hold: bool,
    n_step: u32,
}

pub enum StepError {
    WaitForHoldRequest,
    BadMessage,
}

impl Env {
    pub fn new(
        receiver: Receiver<EnvThreadMessage>,
        game_thread_sender: Sender<GameThreadMessage>,
        plot_thread_sender: Sender<PlotThreadMessage>,
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
            plot_thread_sender,
            pending_transitions: VecDeque::new(),
            truncation_timer: 0,
            waiting_hold,
            n_step: 0,
        })
    }
    fn initial_state_from_frame(frame_1: Rc<ImageOwned2>) -> VecDeque<Rc<ImageOwned2>> {
        let frame_2 = Rc::clone(&frame_1);
        let frame_3 = Rc::clone(&frame_1);
        let frame_4 = Rc::clone(&frame_1);
        VecDeque::from([frame_1, frame_2, frame_3, frame_4])
    }
    fn send_episode_score_to_plot_thread(&self, score: u32) {
        self.plot_thread_sender
            .send(PlotThreadMessage::Datum((
                f64::from(self.n_step),
                f64::from(score),
            )))
            .unwrap();
    }
    pub fn step(&mut self, action: u8) -> Result<(), StepError> {
        let state_slice = Self::state_as_slice(&mut self.state).clone();
        let score = self.score;
        self.game_thread_sender
            .send(GameThreadMessage::Action(action))
            .unwrap();
        let (next_frame, next_score) = self.wait_for_next_frame()?;
        let next_frame = Rc::new(next_frame);
        let terminated = score > next_score;

        if terminated {
            self.send_episode_score_to_plot_thread(score);
        }
        self.n_step += 1;

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
            state_slice,
            next_state_slice,
            action,
            reward,
            terminated,
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
    pub fn pop_transition(&mut self) -> Option<Transition> {
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
