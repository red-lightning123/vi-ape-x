mod message;
mod thread_id;

use super::thread::{ActiveThread, Thread};
use crate::{EnvThread, GameThread, UiThread};
use crate::{EnvThreadMessage, GameThreadMessage, UiThreadMessage};
use crossbeam_channel::Receiver;
pub use message::{MasterMessage, MasterThreadMessage};
use packets::ActorSettings;
use std::fs;
pub use thread_id::ThreadId;

pub enum CommandError {
    ModeMatch,
}

#[derive(Copy, Clone)]
pub enum Mode {
    Running,
    Held,
}

pub struct Master {
    mode: Mode,
    receiver: Receiver<MasterThreadMessage>,
    ui_thread: ActiveThread<UiThread>,
    env_thread: ActiveThread<EnvThread>,
    game_thread: ActiveThread<GameThread>,
}

impl Master {
    pub fn new(settings: ActorSettings) -> Self {
        let (sender, receiver) = crossbeam_channel::unbounded::<MasterThreadMessage>();
        let game_thread = Thread::new();
        let ui_thread = Thread::new();
        let env_thread = Thread::new();

        let game_thread = game_thread.spawn((
            sender.clone(),
            ui_thread.sender().clone(),
            env_thread.sender().clone(),
        ));
        let ui_thread = ui_thread.spawn(sender.clone());
        let env_thread = env_thread.spawn((
            sender,
            ui_thread.sender().clone(),
            game_thread.sender().clone(),
            settings,
        ));
        Self {
            mode: Mode::Held,
            receiver,
            game_thread,
            ui_thread,
            env_thread,
        }
    }

    pub fn mode(&self) -> Mode {
        self.mode
    }

    fn send_all(&self, message: MasterMessage) {
        self.ui_thread.send_master(message.clone()).unwrap();
        self.env_thread.send_master(message.clone()).unwrap();
        self.game_thread.send_master(message).unwrap();
    }

    fn wait_all_done(&self) {
        let mut ready_thread_flags = 0;
        loop {
            match self.receiver.recv().unwrap() {
                MasterThreadMessage::Done(thread_id) => {
                    ready_thread_flags |= thread_id.as_bit_flag();
                    if ready_thread_flags == ThreadId::all_flags() {
                        break;
                    }
                }
            }
        }
    }

    fn wait_env_done(&self) {
        match self.receiver.recv().unwrap() {
            MasterThreadMessage::Done(ThreadId::Env) => {}
            _ => panic!("master thread: bad message"),
        }
    }

    pub fn save(&self) -> Result<(), CommandError> {
        match self.mode {
            Mode::Running => Err(CommandError::ModeMatch),
            Mode::Held => {
                let saved_path = "saved";
                fs::create_dir_all(saved_path).unwrap();
                self.send_all(MasterMessage::Save(saved_path.into()));
                self.wait_all_done();
                Ok(())
            }
        }
    }

    pub fn load(&self) -> Result<(), CommandError> {
        match self.mode {
            Mode::Running => Err(CommandError::ModeMatch),
            Mode::Held => {
                self.send_all(MasterMessage::Load("load".into()));
                self.wait_all_done();
                Ok(())
            }
        }
    }

    pub fn hold(&mut self) -> Result<(), CommandError> {
        match self.mode {
            Mode::Running => {
                self.env_thread.send(EnvThreadMessage::WaitForHold).unwrap();
                self.wait_env_done();
                self.send_all(MasterMessage::PrepareHold);
                self.wait_all_done();
                self.send_all(MasterMessage::Hold);
                self.mode = Mode::Held;
                Ok(())
            }
            Mode::Held => Err(CommandError::ModeMatch),
        }
    }

    pub fn resume(&mut self) -> Result<(), CommandError> {
        match self.mode {
            Mode::Running => Err(CommandError::ModeMatch),
            Mode::Held => {
                self.send_all(MasterMessage::Resume);
                self.mode = Mode::Running;
                // TODO: should probably wait for response
                Ok(())
            }
        }
    }

    pub fn close(self) -> Result<(), (Self, CommandError)> {
        match self.mode {
            Mode::Running => Err((self, CommandError::ModeMatch)),
            Mode::Held => {
                self.send_all(MasterMessage::Close);
                self.ui_thread.join().unwrap();
                self.env_thread.join().unwrap();
                self.game_thread.join().unwrap();
                Ok(())
            }
        }
    }
}
