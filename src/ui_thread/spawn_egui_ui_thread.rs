use crate::{MasterThreadMessage, UiThreadMessage};
use crossbeam_channel::{Receiver, Sender};

pub fn spawn_egui_ui_thread(
    _receiver: Receiver<UiThreadMessage>,
    _master_thread_sender: Sender<MasterThreadMessage>,
) -> std::thread::JoinHandle<()> {
    unimplemented!("egui based ui is not currently supported")
}
