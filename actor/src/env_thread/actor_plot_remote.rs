use model::LearningStepInfo;
use packets::{LearnerPlotKind, PlotKind};
use plot_remote::PlotRemote;
use std::net::SocketAddr;
use std::time::Instant;

pub struct ActorPlotRemote {
    episode_score_plot_remote: PlotRemote,
    start_instant: Instant,
}

impl ActorPlotRemote {
    pub fn new(plot_server_addr: SocketAddr, actor_id: usize, batch_len: usize) -> Self {
        Self {
            episode_score_plot_remote: PlotRemote::new(
                plot_server_addr,
                PlotKind::Actor { id: actor_id },
                batch_len,
            ),
            start_instant: Instant::now(),
        }
    }
    pub fn send(&mut self, episode_score: u32) {
        let secs_since_start = (Instant::now() - self.start_instant).as_secs_f64();
        self.episode_score_plot_remote
            .send((episode_score.into(), secs_since_start));
    }
}
