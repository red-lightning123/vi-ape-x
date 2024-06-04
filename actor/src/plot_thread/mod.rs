mod r#loop;

use crate::master_thread::ThreadType;
use crate::{MasterMessage, MasterThreadMessage};
use crossbeam_channel::{Receiver, Sender};
use r#loop::Loop;
pub use r#loop::{PlotThreadMessage, PlotType};
use std::thread::JoinHandle;

pub struct PlotThread {}

impl ThreadType for PlotThread {
    type Message = PlotThreadMessage;
    type SpawnArgs = Sender<MasterThreadMessage>;

    fn spawn(receiver: Receiver<Self::Message>, args: Self::SpawnArgs) -> JoinHandle<()> {
        std::thread::spawn(move || {
            let master_thread_sender = args;
            Loop::new(receiver, master_thread_sender).run();
        })
    }

    fn master_message(msg: MasterMessage) -> Self::Message {
        Self::Message::Master(msg)
    }
}
