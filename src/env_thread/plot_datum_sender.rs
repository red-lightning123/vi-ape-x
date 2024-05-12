use super::TrainingSchedule;
use crate::{PlotThreadMessage, PlotType};
use crossbeam_channel::Sender;

pub struct PlotDatumSender {
    sender: Sender<PlotThreadMessage>,
}

impl PlotDatumSender {
    pub fn new(sender: Sender<PlotThreadMessage>) -> Self {
        Self { sender }
    }

    fn send_datum(&self, plot_type: PlotType, datum: f64, schedule: &TrainingSchedule) {
        self.sender
            .send(PlotThreadMessage::Datum(
                plot_type,
                (f64::from(schedule.n_step()), datum),
            ))
            .unwrap();
    }

    pub fn send_episode_score(&self, score: u32, schedule: &TrainingSchedule) {
        self.send_datum(PlotType::EpisodeScore, f64::from(score), schedule);
    }

    pub fn send_loss(&self, loss: f32, schedule: &TrainingSchedule) {
        self.send_datum(PlotType::Loss, f64::from(loss), schedule);
    }

    pub fn send_q_val(&self, q_val: f32, schedule: &TrainingSchedule) {
        self.send_datum(PlotType::QVal, f64::from(q_val), schedule);
    }
}
