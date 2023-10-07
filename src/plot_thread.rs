use crate::file_io::{create_file_buf_write, has_data_left, open_file_buf_read};
use crate::{MasterMessage, MasterThreadMessage, ThreadId};
use crossbeam_channel::{Receiver, Sender};
use plotlib::page::Page;
use plotlib::repr::Plot as PlotlibPlot;
use plotlib::style::{LineJoin, LineStyle};
use plotlib::view::ContinuousView;
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
}

impl Plot {
    fn new() -> Self {
        Self {
            points: vec![],
            current_n: 0,
            current_sum: 0.0,
        }
    }
    fn add_datum(&mut self, (x, y): (f64, f64)) {
        self.current_n += 1;
        self.current_sum += y;
        if self.current_n == self.data_per_point() {
            let y_average = self.current_sum / (self.data_per_point() as f64);
            self.points.push((x, y_average));
            self.current_n = 0;
            self.current_sum = 0.0;
            self.draw_plot();
        }
    }
    fn draw_plot(&self) {
        let plot = PlotlibPlot::new(self.points.clone())
            .line_style(LineStyle::new().colour("blue").linejoin(LineJoin::Round));
        let view = ContinuousView::new()
            .add(plot)
            .x_label("Step")
            .y_label("Average Score");
        // an err is typically returned when either the x or y
        // range are invalid. The intended behavior in that case
        // is to ignore the error and just avoid saving the svg.
        // plotlib doesn't seem to provide a convenient way to differentiate
        // between errors for this function, so we can't match for just the
        // range-related error. thus we silently ignore any other errors too
        let _ = Page::single(&view).save("progress.svg");
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
    const fn data_per_point(&self) -> usize {
        const DATA_PER_POINT: usize = 10;
        DATA_PER_POINT
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
