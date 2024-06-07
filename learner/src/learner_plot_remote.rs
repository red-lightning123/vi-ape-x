use model::LearningStepInfo;
use packets::{LearnerPlotKind, PlotKind};
use plot_remote::PlotRemote;
use std::net::SocketAddr;
use std::time::Instant;

pub struct LearnerPlotRemote {
    loss_plot_remote: PlotRemote,
    q_val_plot_remote: PlotRemote,
    start_instant: Instant,
}

impl LearnerPlotRemote {
    pub fn new(plot_server_addr: SocketAddr, batch_len: usize) -> Self {
        Self {
            loss_plot_remote: PlotRemote::new(
                plot_server_addr,
                PlotKind::Learner(LearnerPlotKind::Loss),
                batch_len,
            ),
            q_val_plot_remote: PlotRemote::new(
                plot_server_addr,
                PlotKind::Learner(LearnerPlotKind::QVal),
                batch_len,
            ),
            start_instant: Instant::now(),
        }
    }
    pub fn send(&mut self, step_info: LearningStepInfo) {
        let LearningStepInfo {
            loss,
            average_q_val,
        } = step_info;
        let secs_since_start = (Instant::now() - self.start_instant).as_secs_f64();
        self.loss_plot_remote.send((secs_since_start, loss.into()));
        self.q_val_plot_remote
            .send((secs_since_start, average_q_val.into()));
    }
}
