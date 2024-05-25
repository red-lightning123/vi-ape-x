mod r#loop;

use crate::MasterThreadMessage;
use crossbeam_channel::{Receiver, Sender};
use r#loop::Loop;
pub use r#loop::{PlotThreadMessage, PlotType};

pub fn spawn_plot_thread(
    receiver: Receiver<PlotThreadMessage>,
    master_thread_sender: Sender<MasterThreadMessage>,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        Loop::new(receiver, master_thread_sender).run();
    })
}
