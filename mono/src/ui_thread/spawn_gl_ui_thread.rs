use super::wait_for_hold_message;
use crate::HumanInterface;
use crate::{MasterMessage, MasterThreadMessage, ThreadId, UiThreadMessage};
use crossbeam_channel::{Receiver, Sender};

enum ThreadMode<'a> {
    Running(HumanInterface<'a>),
    Held,
}

pub fn spawn_gl_ui_thread(
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
                            let win = match receiver.recv().unwrap() {
                                UiThreadMessage::WinDims(win) => win,
                                UiThreadMessage::Master(MasterMessage::Hold) => continue,
                                _ => panic!("{THREAD_NAME} thread: bad message"),
                            };
                            let human_interface = HumanInterface::new(&win);
                            mode = ThreadMode::Running(human_interface);
                        }
                        MasterMessage::Close => {
                            break;
                        }
                    },
                    _ => panic!("{THREAD_NAME} thread: bad message"),
                },
                ThreadMode::Running(ref mut human_interface) => {
                    human_interface.clear_window();
                    match receiver.recv().unwrap() {
                        UiThreadMessage::Frame(frame) => {
                            human_interface.set_frame(frame);
                            human_interface.draw();
                            human_interface.swap_buffers();
                        }
                        UiThreadMessage::NStep(n_step) => {
                            human_interface.set_n_step(n_step);
                            human_interface.draw();
                            human_interface.swap_buffers();
                        }
                        UiThreadMessage::Master(MasterMessage::PrepareHold) => {
                            match mode {
                                ThreadMode::Held => unreachable!(),
                                ThreadMode::Running(human_interface) => {
                                    human_interface.terminate();
                                }
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
                    while human_interface.poll_event().is_some() {}
                }
            }
        }
    })
}
