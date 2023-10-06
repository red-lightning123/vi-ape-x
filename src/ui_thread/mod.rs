use crate::ImageOwned2;
use crate::Window;
use crate::{MasterMessage, MasterThreadMessage};
use crossbeam_channel::{Receiver, Sender};
mod spawn_dummy_ui_thread;
use spawn_dummy_ui_thread::spawn_dummy_ui_thread;
mod spawn_gl_ui_thread;
use spawn_gl_ui_thread::spawn_gl_ui_thread;
mod spawn_egui_ui_thread;
use spawn_egui_ui_thread::spawn_egui_ui_thread;

fn wait_for_hold_message(receiver: &Receiver<UiThreadMessage>) {
    loop {
        if let UiThreadMessage::Master(MasterMessage::Hold) = receiver.recv().unwrap() {
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

pub fn spawn_ui_thread(
    receiver: Receiver<UiThreadMessage>,
    master_thread_sender: Sender<MasterThreadMessage>,
) -> std::thread::JoinHandle<()> {
    const VARIANT: UiImplVariant = UiImplVariant::Gl;
    const SPAWN_FN: fn(
        receiver: Receiver<UiThreadMessage>,
        master_thread_sender: Sender<MasterThreadMessage>,
    ) -> std::thread::JoinHandle<()> = match VARIANT {
        UiImplVariant::Dummy => spawn_dummy_ui_thread,
        UiImplVariant::Gl => spawn_gl_ui_thread,
        UiImplVariant::Egui => spawn_egui_ui_thread,
    };
    SPAWN_FN(receiver, master_thread_sender)
}
