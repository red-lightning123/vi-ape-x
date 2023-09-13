use crate::{ MasterThreadMessage, UiThreadMessage };
use crossbeam_channel::{ Sender, Receiver };

pub fn spawn_egui_ui_thread(_receiver : Receiver<UiThreadMessage>, _master_thread_sender : Sender<MasterThreadMessage>) -> std::thread::JoinHandle<()> {
    unimplemented!()
    // egui based ui is not currently supported, the gl variant should be used instead
    // the following code could be adapted into a working implementation, but some changes
    // may be necessary
    /*
    use crate::{ MasterMessage, MasterThreadMessage, ThreadId, UiThreadMessage };
    use crossbeam_channel::{ Sender, Receiver };
    use super::ui_app::UiApp;
    use eframe::egui;
    std::thread::spawn(move || {
        match receiver.recv().unwrap() {
            UiThreadMessage::Master(MasterMessage::Start(_)) => { }
            _ => panic!("ui thread expected MasterMessage::Start but received another message instead")
        }
        let win =
            match receiver.recv().unwrap() {
                UiThreadMessage::WinDims(win) => win,
                UiThreadMessage::Master(message) => {
                    match message {
                        MasterMessage::SaveAndClose => {
                            return;
                        }
                        MasterMessage::Start(_) => {
                                panic!("ui thread received MasterMessage::Start but it is already running");
                        }
                    }
                }
                _ => panic!("ui thread: bad message")
            };
        let options = eframe::NativeOptions {
            initial_window_size : Some(egui::vec2(1920.0, 1080.0)),
            event_loop_builder : Some(Box::new(|builder| { winit::platform::x11::EventLoopBuilderExtX11::with_any_thread(builder, true); })),
            ..Default::default()
        };
        let dims = (u32::from(win.width()), u32::from(win.height()));
        eframe::run_native(
            "Feed",
            options,
            Box::new(move |cc| Box::new(UiApp::new(cc, dims, receiver, master_thread_sender)))
        );
    })
    */
}
