mod spawn_dummy_ui_thread;
mod spawn_egui_ui_thread;
mod spawn_gl_ui_thread;

use crate::master_thread::ThreadType;
use crate::Window;
use crate::{MasterMessage, MasterThreadMessage};
use crossbeam_channel::{Receiver, Sender};
use image::ImageOwned2;
use spawn_dummy_ui_thread::spawn_dummy_ui_thread;
use spawn_egui_ui_thread::spawn_egui_ui_thread;
use spawn_gl_ui_thread::spawn_gl_ui_thread;
use std::thread::JoinHandle;

fn wait_for_hold_message(receiver: &Receiver<UiThreadMessage>) {
    loop {
        if matches!(
            receiver.recv().unwrap(),
            UiThreadMessage::Master(MasterMessage::Hold)
        ) {
            return;
        }
    }
}

pub enum UiThreadMessage {
    WinDims(Window),
    Frame(ImageOwned2),
    NStep(u32),
    Master(MasterMessage),
}

enum UiImplVariant {
    Dummy,
    Gl,
    Egui,
}

pub struct UiThread {}

impl ThreadType for UiThread {
    type Message = UiThreadMessage;
    type SpawnArgs = Sender<MasterThreadMessage>;

    fn spawn(receiver: Receiver<Self::Message>, args: Self::SpawnArgs) -> JoinHandle<()> {
        const VARIANT: UiImplVariant = UiImplVariant::Dummy;
        const SPAWN_FN: fn(
            receiver: Receiver<UiThreadMessage>,
            master_thread_sender: Sender<MasterThreadMessage>,
        ) -> std::thread::JoinHandle<()> = match VARIANT {
            UiImplVariant::Dummy => spawn_dummy_ui_thread,
            UiImplVariant::Gl => spawn_gl_ui_thread,
            UiImplVariant::Egui => spawn_egui_ui_thread,
        };
        let master_thread_sender = args;
        SPAWN_FN(receiver, master_thread_sender)
    }

    fn master_message(msg: MasterMessage) -> Self::Message {
        Self::Message::Master(msg)
    }
}
