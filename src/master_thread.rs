use crate::{ UiThreadMessage, EnvThreadMessage, GameThreadMessage, PlotThreadMessage };
use crate::{ spawn_game_thread, spawn_ui_thread, spawn_plot_thread, spawn_env_thread };
use crossbeam_channel::{ Sender, Receiver };
use std::path::{ Path, PathBuf };
use std::thread::JoinHandle;

#[derive(Clone, Debug)]
pub enum MasterMessage {
    Save(PathBuf),
    Load(PathBuf),
    PrepareHold,
    Hold,
    Resume,
    Close
}

struct MasterSender {
    ui_thread_sender : Sender<UiThreadMessage>,
    env_thread_sender : Sender<EnvThreadMessage>,
    game_thread_sender : Sender<GameThreadMessage>,
    plot_thread_sender : Sender<PlotThreadMessage>
}

impl MasterSender {
    fn send_all(&self, message : MasterMessage) {
        self.ui_thread_sender.send(UiThreadMessage::Master(message.clone())).unwrap();
        self.env_thread_sender.send(EnvThreadMessage::Master(message.clone())).unwrap();
        self.game_thread_sender.send(GameThreadMessage::Master(message.clone())).unwrap();
        self.plot_thread_sender.send(PlotThreadMessage::Master(message)).unwrap();
    }
}


fn wait_all_done(receiver : &Receiver<MasterThreadMessage>) {
    let mut ready_thread_flags = 0;
    loop {
        match receiver.recv().unwrap() {
            MasterThreadMessage::Done(thread_id) => {
                ready_thread_flags |= thread_id.as_bit_flag();
                if ready_thread_flags == ThreadId::all_flags() {
                    break;
                }
            }
            _ => panic!("master thread: bad message")
        }
    }
}

fn wait_env_done(receiver : &Receiver<MasterThreadMessage>) {
    match receiver.recv().unwrap() {
        MasterThreadMessage::Done(ThreadId::Env) => { }
        _ => panic!("master thread: bad message")
    }
}

pub enum Query {
    NStep
}

fn query_try_from_str(s : &str) -> Option<Query> {
    if s == "n_step" {
        Some(Query::NStep)
    } else {
        None
    }
}

pub enum ThreadId {
    Game,
    Ui,
    Plot,
    Env
}

impl ThreadId {
    fn as_bit_flag(&self) -> u8 {
        match self {
            ThreadId::Game => 1,
            ThreadId::Ui => 2,
            ThreadId::Plot => 4,
            ThreadId::Env => 8
        }
    }
    fn all_flags() -> u8 {
        ThreadId::Game.as_bit_flag() | ThreadId::Ui.as_bit_flag() | ThreadId::Plot.as_bit_flag() | ThreadId::Env.as_bit_flag()
    }
}

pub enum MasterThreadMessage {
    Done(ThreadId),
    QueryReply(String)
}

enum ThreadMode {
    Running,
    Held
}

fn save(master_sender : &MasterSender, receiver : &Receiver<MasterThreadMessage>) {
    master_sender.send_all(MasterMessage::Save(Path::new("saved").to_path_buf()));
    wait_all_done(receiver);
}

fn load(master_sender : &MasterSender, receiver : &Receiver<MasterThreadMessage>) {
    master_sender.send_all(MasterMessage::Load(Path::new("load").to_path_buf()));
    wait_all_done(receiver);
}

fn hold(master_sender : &MasterSender, receiver : &Receiver<MasterThreadMessage>, env_thread_sender : &Sender<EnvThreadMessage>, mode : &mut ThreadMode) {
    env_thread_sender.send(EnvThreadMessage::WaitForHold).unwrap();
    wait_env_done(receiver);
    master_sender.send_all(MasterMessage::PrepareHold);
    wait_all_done(receiver);
    master_sender.send_all(MasterMessage::Hold);
    *mode = ThreadMode::Held;
}

fn resume(master_sender : &MasterSender, mode : &mut ThreadMode) {
    master_sender.send_all(MasterMessage::Resume);
    *mode = ThreadMode::Running;
    // TODO: should probably wait for response
}

fn close(master_sender : &MasterSender, game_thread : JoinHandle<()>, ui_thread : JoinHandle<()>, plot_thread : JoinHandle<()>, env_thread : JoinHandle<()>) {
    master_sender.send_all(MasterMessage::Close);
    game_thread.join().unwrap();
    ui_thread.join().unwrap();
    plot_thread.join().unwrap();
    env_thread.join().unwrap();
}

pub fn spawn_master_thread() -> JoinHandle<()> {
    std::thread::spawn(move || {
        const THREAD_NAME : &str = "master";
        let (sender, receiver) = crossbeam_channel::unbounded::<MasterThreadMessage>();
        let (ui_thread_sender, ui_thread_receiver) = crossbeam_channel::unbounded::<UiThreadMessage>();
        let (env_thread_sender, env_thread_receiver) = crossbeam_channel::unbounded::<EnvThreadMessage>();
        let (env_thread_query_sender, env_thread_query_receiver) = crossbeam_channel::unbounded::<Query>();
        let (game_thread_sender, game_thread_receiver) = crossbeam_channel::unbounded::<GameThreadMessage>();
        let (plot_thread_sender, plot_thread_receiver) = crossbeam_channel::unbounded::<PlotThreadMessage>();
        let master_sender = MasterSender {
            ui_thread_sender: ui_thread_sender.clone(),
            env_thread_sender: env_thread_sender.clone(),
            game_thread_sender: game_thread_sender.clone(),
            plot_thread_sender: plot_thread_sender.clone()
        };
        let mut mode = ThreadMode::Held;
        
        let game_thread = spawn_game_thread(game_thread_receiver, sender.clone(), ui_thread_sender.clone(), env_thread_sender.clone());
        let ui_thread = spawn_ui_thread(ui_thread_receiver, sender.clone());
        let plot_thread = spawn_plot_thread(plot_thread_receiver, sender.clone());
        let env_thread = spawn_env_thread(env_thread_receiver, env_thread_query_receiver, sender, ui_thread_sender, game_thread_sender, plot_thread_sender);

        loop {
            let mut command = String::new();
            std::io::stdin().read_line(&mut command).unwrap();
            let command = command.split_whitespace().collect::<Vec<_>>();
            match command[..] {
                ["save"] => {
                    match mode {
                        ThreadMode::Held => save(&master_sender, &receiver),
                        ThreadMode::Running => eprintln!("command cannot be executed in running mode")
                    }
                }
                ["load"] => {
                    match mode {
                        ThreadMode::Held => load(&master_sender, &receiver),
                        ThreadMode::Running => eprintln!("command cannot be executed in running mode")
                    }
                }
                ["hold"] => {
                    match mode {
                        ThreadMode::Held => eprintln!("command cannot be executed in hold mode"),
                        ThreadMode::Running => hold(&master_sender, &receiver, &env_thread_sender, &mut mode)
                    }
                }
                ["resume"] => {
                    match mode {
                        ThreadMode::Held => resume(&master_sender, &mut mode),
                        ThreadMode::Running => eprintln!("command cannot be executed in running mode")
                    }
                }
                ["close"] => {
                    match mode {
                        ThreadMode::Held => { close(&master_sender, game_thread, ui_thread, plot_thread, env_thread); break; },
                        ThreadMode::Running => eprintln!("command cannot be executed in running mode")
                    }
                }
                ["query", val_name] => {
                    if let Some(query) = query_try_from_str(val_name) {
                        env_thread_query_sender.send(query).unwrap();
                        match receiver.recv().unwrap() {
                            MasterThreadMessage::QueryReply(string) => {
                                println!("{}", string);
                            }
                            _ => panic!("{THREAD_NAME} thread: bad message")
                        }
                    } else {
                        println!("invalid name given to query command");
                    }
                }
                _ => { println!("invalid command"); }
            }
        }
    })
}
