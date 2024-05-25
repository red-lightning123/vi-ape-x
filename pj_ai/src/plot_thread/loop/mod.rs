mod plot_set;

use crate::{MasterMessage, MasterThreadMessage, ThreadId};
use crossbeam_channel::{Receiver, Sender};
pub use plot_set::{PlotSet, PlotType};
use std::fs;
use std::path::Path;

pub enum PlotThreadMessage {
    Datum(PlotType, (f64, f64)),
    Master(MasterMessage),
}

enum ThreadMode {
    Running,
    Held,
}

pub struct Loop {
    receiver: Receiver<PlotThreadMessage>,
    master_thread_sender: Sender<MasterThreadMessage>,
    plots: PlotSet,
}

impl Loop {
    pub fn new(
        receiver: Receiver<PlotThreadMessage>,
        master_thread_sender: Sender<MasterThreadMessage>,
    ) -> Self {
        Self {
            receiver,
            master_thread_sender,
            plots: PlotSet::new("progress"),
        }
    }
    pub fn run(&mut self) {
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
