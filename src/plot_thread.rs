use crate::file_io::{create_file_buf_write, has_data_left, open_file_buf_read};
use crate::{MasterMessage, MasterThreadMessage, ThreadId};
use crossbeam_channel::{Receiver, Sender};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

pub enum PlotThreadMessage {
    Datum(PlotType, (f64, f64)),
    Master(MasterMessage),
}

enum ThreadMode {
    Running,
    Held,
}

#[derive(Serialize, Deserialize)]
struct Plot {
    points: Vec<(f64, f64)>,
    current_n: usize,
    current_sum: f64,
    data_per_point: usize,
    output_path: PathBuf,
    fs_name: PathBuf,
}

impl Plot {
    fn new(output_path: PathBuf, fs_name: PathBuf, data_per_point: usize) -> Self {
        const DATA_PER_POINT: usize = 10;
        Self {
            points: vec![],
            current_n: 0,
            current_sum: 0.0,
            data_per_point,
            output_path,
            fs_name,
        }
    }
    fn add_datum(&mut self, (x, y): (f64, f64)) {
        self.current_n += 1;
        self.current_sum += y;
        if self.current_n == self.data_per_point {
            let y_average = self.current_sum / (self.data_per_point as f64);
            self.points.push((x, y_average));
            self.current_n = 0;
            self.current_sum = 0.0;
            self.update_plot();
        }
    }
    fn update_plot(&self) {
        // The plot is meant to be used in a data-science context. It
        // is therefore desirable for external tools to be able to
        // analyze the plot as they see fit, possibly producing several
        // images from the same data.
        // As such, we do not export to a visual format like svg or
        // draw to the screen, which would heavily constrain the types
        // of analysis an external tool could do.
        // Instead, it seems better to serialize the plot and let the
        // external viewer decide what it wants to do with the data.
        // As for the chosen serialization format, json lends itself
        // quite naturally. Being simple, readable, and
        // self-documenting, it is an ideal format for basic analysis
        self.export_json();
    }
    fn export_json(&self) {
        let file =
            create_file_buf_write(self.output_path.join(&self.fs_name).with_extension("json"))
                .unwrap();
        serde_json::to_writer(file, self).unwrap();
    }
    fn save<P: AsRef<Path>>(&self, path: P) {
        let file = create_file_buf_write(path).unwrap();
        bincode::serialize_into(file, self).unwrap();
    }
    fn load<P: AsRef<Path>>(&mut self, path: P) {
        let mut file = open_file_buf_read(path).unwrap();
        *self = bincode::deserialize_from(&mut file).unwrap();
        assert!(
            !has_data_left(file).unwrap(),
            "deserialization of file didn't reach EOF"
        );
    }
    fn fs_name(&self) -> &PathBuf {
        &self.fs_name
    }
}

pub enum PlotType {
    EpisodeScore,
}

struct PlotSet {
    episode_score: Plot,
}

impl PlotSet {
    fn new<P: AsRef<Path>>(output_path: P) -> Self {
        let output_path = output_path.as_ref();
        Self {
            episode_score: Plot::new(output_path.into(), "episode_score".into(), 10),
        }
    }
    fn add_datum(&mut self, plot_type: PlotType, datum: (f64, f64)) {
        self.plot_mut(plot_type).add_datum(datum);
    }
    fn save<P: AsRef<Path>>(&self, path: P) {
        let path = path.as_ref();
        for plot in self.plots() {
            plot.save(path.join(plot.fs_name()));
        }
    }
    fn load<P: AsRef<Path>>(&mut self, path: P) {
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

struct Loop {
    receiver: Receiver<PlotThreadMessage>,
    master_thread_sender: Sender<MasterThreadMessage>,
    plots: PlotSet,
}

impl Loop {
    fn new(
        receiver: Receiver<PlotThreadMessage>,
        master_thread_sender: Sender<MasterThreadMessage>,
    ) -> Self {
        Self {
            receiver,
            master_thread_sender,
            plots: PlotSet::new("progress"),
        }
    }
    fn run(&mut self) {
        const THREAD_ID: ThreadId = ThreadId::Plot;
        const THREAD_NAME: &str = "plot";
        let mut mode = ThreadMode::Held;
        loop {
            match mode {
                ThreadMode::Held => match self.receiver.recv().unwrap() {
                    PlotThreadMessage::Master(message) => match message {
                        MasterMessage::Save(path) => {
                            self.save(path);
                            self.master_thread_sender
                                .send(MasterThreadMessage::Done(THREAD_ID))
                                .unwrap();
                        }
                        MasterMessage::Load(path) => {
                            self.load(path);
                            self.master_thread_sender
                                .send(MasterThreadMessage::Done(THREAD_ID))
                                .unwrap();
                        }
                        message @ (MasterMessage::Hold | MasterMessage::PrepareHold) => {
                            eprintln!("{THREAD_NAME} thread: {:?} while already held", message);
                        }
                        MasterMessage::Resume => {
                            mode = ThreadMode::Running;
                        }
                        MasterMessage::Close => {
                            break;
                        }
                    },
                    _ => panic!("{THREAD_NAME} thread: bad message"),
                },
                ThreadMode::Running => match self.receiver.recv().unwrap() {
                    PlotThreadMessage::Datum(plot_type, datum) => {
                        self.plots.add_datum(plot_type, datum);
                    }
                    PlotThreadMessage::Master(message) => match message {
                        MasterMessage::PrepareHold => {
                            self.master_thread_sender
                                .send(MasterThreadMessage::Done(THREAD_ID))
                                .unwrap();
                        }
                        MasterMessage::Hold => {
                            mode = ThreadMode::Held;
                        }
                        _ => panic!("{THREAD_NAME} thread: bad message"),
                    },
                },
            }
        }
    }
    fn save<P: AsRef<Path>>(&self, path: P) {
        let plots_path = path.as_ref().join("plots");
        fs::create_dir_all(&plots_path).unwrap();
        self.plots.save(plots_path);
    }
    fn load<P: AsRef<Path>>(&mut self, path: P) {
        self.plots.load(path.as_ref().join("plots"));
    }
}

pub fn spawn_plot_thread(
    receiver: Receiver<PlotThreadMessage>,
    master_thread_sender: Sender<MasterThreadMessage>,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        Loop::new(receiver, master_thread_sender).run();
    })
}
