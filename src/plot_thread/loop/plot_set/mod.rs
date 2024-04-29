mod plot;
use plot::Plot;
use std::path::Path;

pub enum PlotType {
    EpisodeScore,
}

pub struct PlotSet {
    episode_score: Plot,
}

impl PlotSet {
    pub fn new<P: AsRef<Path>>(output_path: P) -> Self {
        let output_path = output_path.as_ref();
        Self {
            episode_score: Plot::new(output_path.into(), "episode_score".into(), 10),
        }
    }
    pub fn add_datum(&mut self, plot_type: PlotType, datum: (f64, f64)) {
        self.plot_mut(plot_type).add_datum(datum);
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
    fn plot_mut(&mut self, plot_type: PlotType) -> &mut Plot {
        match plot_type {
            PlotType::EpisodeScore => &mut self.episode_score,
        }
    }
    fn plots(&self) -> [&Plot; 1] {
        [&self.episode_score]
    }
    fn plots_mut(&mut self) -> [&mut Plot; 1] {
        [&mut self.episode_score]
    }
}
