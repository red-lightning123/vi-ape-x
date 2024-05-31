use super::BasicEpisode;
use super::{Done, StateAccum, Status};
use replay_data::GenericTransition;
use std::collections::VecDeque;

pub struct TimeLimitedWrapper<State>
where
    State: StateAccum,
{
    episode: BasicEpisode<State>,
    truncation_timer: u32,
    current_score_record: u32,
}

impl<State> TimeLimitedWrapper<State>
where
    State: StateAccum,
    <State as StateAccum>::View: Clone,
{
    pub fn new(episode: BasicEpisode<State>) -> Self {
        let current_score_record = episode.score();
        Self {
            episode,
            truncation_timer: 0,
            current_score_record,
        }
    }
    pub fn step(
        &mut self,
        action: u8,
        next_frame: <State as StateAccum>::Frame,
        next_score: u32,
        transition_queue: &mut VecDeque<(GenericTransition<State::View>, Option<u32>)>,
    ) -> Status {
        let score_exceeded_record = self.receive_next_score(next_score);
        if score_exceeded_record {
            self.truncation_timer = 0;
        } else {
            self.truncation_timer += 1;
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
        self.current_score_record = self.episode.score();
    }
    fn receive_next_score(&mut self, next_score: u32) -> bool {
        let score_exceeded_record = self.current_score_record < next_score;
        if score_exceeded_record {
            self.current_score_record = next_score;
        }
        score_exceeded_record
    }
    fn truncation_timer_exceeded_threshold(&self) -> bool {
        const TIMER_THRESHOLD: u32 = 200;
        self.truncation_timer >= TIMER_THRESHOLD
    }
    pub fn state(&self) -> State::View {
        self.episode.state()
    }
}
