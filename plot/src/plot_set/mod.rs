mod plot;

use packets::{LearnerPlotKind, PlotKind};
use plot::Plot;
use std::path::Path;

pub struct PlotSet {
    actor_episode_scores: Vec<Plot>,
    loss: Plot,
    q_val: Plot,
}

impl PlotSet {
    pub fn new<P: AsRef<Path>>(output_path: P, actor_count: usize) -> Self {
        let output_path = output_path.as_ref();
        Self {
            actor_episode_scores: (0..actor_count)
                .map(|actor_id| {
                    Plot::new(output_path.join("actors"), actor_id.to_string().into(), 1)
                })
                .collect(),
            loss: Plot::new(output_path.into(), "loss".into(), 2500),
            q_val: Plot::new(output_path.into(), "q_val".into(), 2500),
        }
    }
    pub fn add_datum(&mut self, plot_kind: PlotKind, datum: (f64, f64)) {
        self.plot_mut(plot_kind).add_datum(datum);
    }
    pub fn save<P: AsRef<Path>>(&self, path: P) {
        let path = path.as_ref();
        for plot in self.plots() {
            plot.save(path.join(plot.fs_name()));
        }
    }
    pub fn load<P: AsRef<Path>>(&mut self, path: P) {
        let path = path.as_ref();
        for plot in self.plots_mut() {
            plot.load(path.join(plot.fs_name()));
        }
    }
    fn plot_mut(&mut self, kind: PlotKind) -> &mut Plot {
        match kind {
            PlotKind::Actor { id } => &mut self.actor_episode_scores[id],
            PlotKind::Learner(learner_kind) => match learner_kind {
                LearnerPlotKind::Loss => &mut self.loss,
                LearnerPlotKind::QVal => &mut self.q_val,
            },
        }
    }
    fn plots(&self) -> [&Plot; 2] {
        [&self.loss, &self.q_val]
    }
    fn plots_mut(&mut self) -> [&mut Plot; 2] {
        [&mut self.loss, &mut self.q_val]
    }
}
