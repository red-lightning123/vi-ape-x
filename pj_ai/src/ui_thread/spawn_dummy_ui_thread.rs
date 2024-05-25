use super::wait_for_hold_message;
use crate::{MasterMessage, MasterThreadMessage, ThreadId, UiThreadMessage};
use crossbeam_channel::{Receiver, Sender};

enum ThreadMode {
    Running,
    Held,
}

pub fn spawn_dummy_ui_thread(
    receiver: Receiver<UiThreadMessage>,
    master_thread_sender: Sender<MasterThreadMessage>,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        const THREAD_ID: ThreadId = ThreadId::Ui;
        const THREAD_NAME: &str = "ui";
        let mut mode = ThreadMode::Held;
        loop {
            match mode {
                ThreadMode::Held => match receiver.recv().unwrap() {
                    UiThreadMessage::Master(message) => match message {
                        MasterMessage::Save(_) => {
                            master_thread_sender
                                .send(MasterThreadMessage::Done(THREAD_ID))
                                .unwrap();
                        }
                        MasterMessage::Load(_) => {
                            master_thread_sender
                                .send(MasterThreadMessage::Done(THREAD_ID))
                                .unwrap();
                        }
                        message @ (MasterMessage::Hold | MasterMessage::PrepareHold) => {
                            eprintln!("{THREAD_NAME} thread: {:?} while already held", message);
                        }
                        MasterMessage::Resume => {
                            match receiver.recv().unwrap() {
                                UiThreadMessage::WinDims(_win) => {}
                                UiThreadMessage::Master(MasterMessage::Hold) => continue,
                                _ => panic!("{THREAD_NAME} thread: bad message"),
                            };
                            mode = ThreadMode::Running;
                        }
                        MasterMessage::Close => {
                            break;
                        }
                    },
                    _ => panic!("{THREAD_NAME} thread: bad message"),
                },
                ThreadMode::Running => {
                    match receiver.recv().unwrap() {
                        UiThreadMessage::Frame(_frame) => {}
                        UiThreadMessage::NStep(_n_step) => {}
                        UiThreadMessage::Master(MasterMessage::PrepareHold) => {
                            match mode {
                                ThreadMode::Held => unreachable!(),
                                ThreadMode::Running => {}
                            }
                            mode = ThreadMode::Held;
                            master_thread_sender
                                .send(MasterThreadMessage::Done(THREAD_ID))
                                .unwrap();
                            wait_for_hold_message(&receiver);
                            continue;
                        }
                        _ => panic!("{THREAD_NAME} thread: bad message"),
                    };
                }
            }
        }
    })
}
