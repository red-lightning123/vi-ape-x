mod master;
mod thread;

use master::{CommandError, Master, Mode};
pub use master::{MasterMessage, MasterThreadMessage, ThreadId};
use packets::ActorSettings;
use std::thread::JoinHandle;
pub use thread::ThreadType;

fn print_mode_match_err(mode: Mode) {
    let mode = match mode {
        Mode::Running => "running",
        Mode::Held => "hold",
    };
    eprintln!("command cannot be executed in {} mode", mode)
}

pub fn spawn_master_thread(args: crate::Args, settings: ActorSettings) -> JoinHandle<()> {
    std::thread::spawn(move || {
        const THREAD_NAME: &str = "master";
        let mut master = Master::new(args, settings.clone());

        if settings.activate {
            master.resume().unwrap_or_else(|e| match e {
                CommandError::ModeMatch => unreachable!("the master should start in Mode::Held"),
            });
        }

        loop {
            let mut command = String::new();
            std::io::stdin().read_line(&mut command).unwrap();
            let command = command.split_whitespace().collect::<Vec<_>>();
            match command[..] {
                ["save", path] => master.save(path).unwrap_or_else(|e| match e {
                    CommandError::ModeMatch => print_mode_match_err(master.mode()),
                }),
                ["load", path] => master.load(path).unwrap_or_else(|e| match e {
                    CommandError::ModeMatch => print_mode_match_err(master.mode()),
                }),
                ["hold"] => master.hold().unwrap_or_else(|e| match e {
                    CommandError::ModeMatch => print_mode_match_err(master.mode()),
                }),
                ["resume"] => master.resume().unwrap_or_else(|e| match e {
                    CommandError::ModeMatch => print_mode_match_err(master.mode()),
                }),
                ["close"] => match master.close() {
                    Ok(()) => break,
                    Err((master_return, CommandError::ModeMatch)) => {
                        master = master_return;
                        print_mode_match_err(master.mode());
                    }
                },
                _ => {
                    println!("invalid command");
                }
            }
        }
    })
}
