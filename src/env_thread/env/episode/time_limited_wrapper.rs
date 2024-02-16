use super::BasicEpisode;
use super::{Done, Status};
use super::{State, Transition};
use crate::ImageOwned2;
use std::collections::VecDeque;

pub struct TimeLimitedWrapper {
    episode: BasicEpisode,
    truncation_timer: u32,
}

impl TimeLimitedWrapper {
    pub fn new(episode: BasicEpisode) -> Self {
        Self {
            episode,
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
        if self.episode.score() == next_score {
            self.truncation_timer += 1;
        } else {
            self.truncation_timer = 0;
        }
        let status = self
            .episode
            .step(action, next_frame, next_score, transition_queue);
        match status {
            Status::Done(Done::Terminated) => {
                self.reset_wrapper_to_current();
                status
            }
            Status::Running if self.truncation_timer_exceeded_threshold() => {
                Status::Done(Done::ShouldTruncate)
            }
            Status::Running | Status::Done(Done::ShouldTruncate) => status,
        }
    }
    fn reset_wrapper_to_current(&mut self) {
        self.truncation_timer = 0;
    }
    fn truncation_timer_exceeded_threshold(&self) -> bool {
        const TIMER_THRESHOLD: u32 = 200;
        self.truncation_timer >= TIMER_THRESHOLD
    }
    pub fn state(&self) -> State {
        self.episode.state()
    }
}
