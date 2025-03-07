use crate::{spawn_env_thread, spawn_game_thread, spawn_plot_thread, spawn_ui_thread};
use crate::{EnvThreadMessage, GameThreadMessage, PlotThreadMessage, UiThreadMessage};
use crossbeam_channel::{Receiver, Sender};
use std::fs;
use std::path::PathBuf;
use std::thread::JoinHandle;

#[derive(Clone, Debug)]
pub enum MasterMessage {
    Save(PathBuf),
    Load(PathBuf),
    PrepareHold,
    Hold,
    Resume,
    Close,
}

struct MasterSender {
    ui_thread_sender: Sender<UiThreadMessage>,
    env_thread_sender: Sender<EnvThreadMessage>,
    game_thread_sender: Sender<GameThreadMessage>,
    plot_thread_sender: Sender<PlotThreadMessage>,
}

impl MasterSender {
    fn send_all(&self, message: MasterMessage) {
        self.ui_thread_sender
            .send(UiThreadMessage::Master(message.clone()))
            .unwrap();
        self.env_thread_sender
            .send(EnvThreadMessage::Master(message.clone()))
            .unwrap();
        self.game_thread_sender
            .send(GameThreadMessage::Master(message.clone()))
            .unwrap();
        self.plot_thread_sender
            .send(PlotThreadMessage::Master(message))
            .unwrap();
    }
}

fn wait_all_done(receiver: &Receiver<MasterThreadMessage>) {
    let mut ready_thread_flags = 0;
    loop {
        match receiver.recv().unwrap() {
            MasterThreadMessage::Done(thread_id) => {
                ready_thread_flags |= thread_id.as_bit_flag();
                if ready_thread_flags == ThreadId::all_flags() {
                    break;
                }
            }
        }
    }
}

fn wait_env_done(receiver: &Receiver<MasterThreadMessage>) {
    match receiver.recv().unwrap() {
        MasterThreadMessage::Done(ThreadId::Env) => {}
        _ => panic!("master thread: bad message"),
    }
}

pub enum ThreadId {
    Game,
    Ui,
    Plot,
    Env,
}

impl ThreadId {
    fn as_bit_flag(&self) -> u8 {
        match self {
            Self::Game => 1,
            Self::Ui => 2,
            Self::Plot => 4,
            Self::Env => 8,
        }
    }
    fn all_flags() -> u8 {
        Self::Game.as_bit_flag()
            | Self::Ui.as_bit_flag()
            | Self::Plot.as_bit_flag()
            | Self::Env.as_bit_flag()
    }
}

pub enum MasterThreadMessage {
    Done(ThreadId),
}

enum ThreadMode {
    Running,
    Held,
}

fn save(master_sender: &MasterSender, receiver: &Receiver<MasterThreadMessage>) {
    let saved_path = "saved";
    fs::create_dir_all(saved_path).unwrap();
    master_sender.send_all(MasterMessage::Save(saved_path.into()));
    wait_all_done(receiver);
}

fn load(master_sender: &MasterSender, receiver: &Receiver<MasterThreadMessage>) {
    master_sender.send_all(MasterMessage::Load("load".into()));
    wait_all_done(receiver);
}

fn hold(
    master_sender: &MasterSender,
    receiver: &Receiver<MasterThreadMessage>,
    env_thread_sender: &Sender<EnvThreadMessage>,
    mode: &mut ThreadMode,
) {
    env_thread_sender
        .send(EnvThreadMessage::WaitForHold)
        .unwrap();
    wait_env_done(receiver);
    master_sender.send_all(MasterMessage::PrepareHold);
    wait_all_done(receiver);
    master_sender.send_all(MasterMessage::Hold);
    *mode = ThreadMode::Held;
}

fn resume(master_sender: &MasterSender, mode: &mut ThreadMode) {
    master_sender.send_all(MasterMessage::Resume);
    *mode = ThreadMode::Running;
    // TODO: should probably wait for response
}

fn close(
    master_sender: &MasterSender,
    game_thread: JoinHandle<()>,
    ui_thread: JoinHandle<()>,
    plot_thread: JoinHandle<()>,
    env_thread: JoinHandle<()>,
) {
    master_sender.send_all(MasterMessage::Close);
    game_thread.join().unwrap();
    ui_thread.join().unwrap();
    plot_thread.join().unwrap();
    env_thread.join().unwrap();
}

pub fn spawn_master_thread() -> JoinHandle<()> {
    std::thread::spawn(move || {
        const THREAD_NAME: &str = "master";
        let (sender, receiver) = crossbeam_channel::unbounded::<MasterThreadMessage>();
        let (ui_thread_sender, ui_thread_receiver) =
            crossbeam_channel::unbounded::<UiThreadMessage>();
        let (env_thread_sender, env_thread_receiver) =
            crossbeam_channel::unbounded::<EnvThreadMessage>();
        let (game_thread_sender, game_thread_receiver) =
            crossbeam_channel::unbounded::<GameThreadMessage>();
        let (plot_thread_sender, plot_thread_receiver) =
            crossbeam_channel::unbounded::<PlotThreadMessage>();
        let master_sender = MasterSender {
            ui_thread_sender: ui_thread_sender.clone(),
            env_thread_sender: env_thread_sender.clone(),
            game_thread_sender: game_thread_sender.clone(),
            plot_thread_sender: plot_thread_sender.clone(),
        };
        let mut mode = ThreadMode::Held;

        let game_thread = spawn_game_thread(
            game_thread_receiver,
            sender.clone(),
            ui_thread_sender.clone(),
            env_thread_sender.clone(),
        );
        let ui_thread = spawn_ui_thread(ui_thread_receiver, sender.clone());
        let plot_thread = spawn_plot_thread(plot_thread_receiver, sender.clone());
        let env_thread = spawn_env_thread(
            env_thread_receiver,
            sender,
            ui_thread_sender,
            game_thread_sender,
            plot_thread_sender,
        );

        loop {
            let mut command = String::new();
            std::io::stdin().read_line(&mut command).unwrap();
            let command = command.split_whitespace().collect::<Vec<_>>();
            match command[..] {
                ["save"] => match mode {
                    ThreadMode::Held => save(&master_sender, &receiver),
                    ThreadMode::Running => eprintln!("command cannot be executed in running mode"),
                },
                ["load"] => match mode {
                    ThreadMode::Held => load(&master_sender, &receiver),
                    ThreadMode::Running => eprintln!("command cannot be executed in running mode"),
                },
                ["hold"] => match mode {
                    ThreadMode::Held => eprintln!("command cannot be executed in hold mode"),
                    ThreadMode::Running => {
                        hold(&master_sender, &receiver, &env_thread_sender, &mut mode)
                    }
                },
                ["resume"] => match mode {
                    ThreadMode::Held => resume(&master_sender, &mut mode),
                    ThreadMode::Running => eprintln!("command cannot be executed in running mode"),
                },
                ["close"] => match mode {
                    ThreadMode::Held => {
                        close(
                            &master_sender,
                            game_thread,
                            ui_thread,
                            plot_thread,
                            env_thread,
                        );
                        break;
                    }
                    ThreadMode::Running => eprintln!("command cannot be executed in running mode"),
                },
                _ => {
                    println!("invalid command");
                }
            }
        }
    })
}
