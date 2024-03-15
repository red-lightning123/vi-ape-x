use crate::file_io::{create_file_buf_write, has_data_left, open_file_buf_read};
use crate::{MasterMessage, MasterThreadMessage, ThreadId};
use crossbeam_channel::{Receiver, Sender};
use serde::{Deserialize, Serialize};
use std::path::Path;

pub enum PlotThreadMessage {
    Datum((f64, f64)),
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
}

impl Plot {
    fn new() -> Self {
        const DATA_PER_POINT: usize = 10;
        Self {
            points: vec![],
            current_n: 0,
            current_sum: 0.0,
            data_per_point: DATA_PER_POINT,
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
        self.export_json("progress.json");
    }
    fn export_json<P: AsRef<Path>>(&self, path: P) {
        let file = create_file_buf_write(path).unwrap();
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
}

struct Loop {
    receiver: Receiver<PlotThreadMessage>,
    master_thread_sender: Sender<MasterThreadMessage>,
    plot: Plot,
}

impl Loop {
    fn new(
        receiver: Receiver<PlotThreadMessage>,
        master_thread_sender: Sender<MasterThreadMessage>,
    ) -> Self {
        Self {
            receiver,
            master_thread_sender,
            plot: Plot::new(),
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
                    PlotThreadMessage::Datum(datum) => {
                        self.plot.add_datum(datum);
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
        self.plot.save(path.as_ref().join("plot"));
    }
    fn load<P: AsRef<Path>>(&mut self, path: P) {
        self.plot.load(path.as_ref().join("plot"));
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
