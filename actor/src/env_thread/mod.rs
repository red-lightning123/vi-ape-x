mod actor_plot_remote;
mod actor_schedule;
mod env;
mod param_updater_thread;
mod state_accums;

use crate::master_thread::ThreadType;
use crate::{GameThreadMessage, MasterMessage, MasterThreadMessage, ThreadId, UiThreadMessage};
use actor_plot_remote::ActorPlotRemote;
use actor_schedule::ActorSchedule;
use crossbeam_channel::{Receiver, Sender};
use env::{Env, StepError};
use image::ImageOwned2;
use model::traits::{Actor, Persistable, TargetNet};
use model::BasicModel;
use packets::ActorSettings;
use param_updater_thread::{spawn_param_updater_thread, ParamUpdaterThreadMessage};
use rand::Rng;
use replay_data::State;
use replay_wrappers::RemoteReplayWrapper;
use state_accums::filters::{CompressFilter, Filter};
use state_accums::{FrameStack, PipeFilterToAccum};
use std::sync::{Arc, RwLock};
use std::thread::JoinHandle;

type Accum = PipeFilterToAccum<CompressFilter, FrameStack<<CompressFilter as Filter>::Output>>;
type ConcreteEnv = Env<Accum>;

fn random_action() -> u8 {
    rand::thread_rng().gen_range(0..ConcreteEnv::n_actions())
}

const THREAD_ID: ThreadId = ThreadId::Env;
const THREAD_NAME: &str = "env";

fn step(
    env: &mut ConcreteEnv,
    agent: &Arc<RwLock<RemoteReplayWrapper<BasicModel>>>,
    schedule: &mut ActorSchedule,
    master_thread_sender: &Sender<MasterThreadMessage>,
    ui_thread_sender: &Sender<UiThreadMessage>,
    plot_remote: &mut Option<ActorPlotRemote>,
    param_updater_thread_sender: &Sender<ParamUpdaterThreadMessage>,
) -> bool {
    let state = env.state();
    let concated_state = State::concat_frames(&(&state).into());
    ui_thread_sender
        .send(UiThreadMessage::Frame(concated_state))
        .unwrap();
    ui_thread_sender
        .send(UiThreadMessage::NStep(schedule.n_step()))
        .unwrap();
    let action = if rand::thread_rng().gen::<f64>() < schedule.eps() {
        random_action()
    } else {
        let agent = agent.read().unwrap();
        agent.best_action(&state)
    };
    // the env thread needs to handle hold requests carefully
    // the purpose of this variable is to ensure that the env thread
    // obeys hold requests right at the end of the frame where they
    // were produced
    let mut should_hold = false;
    match env.step(action) {
        Ok(()) => {}
        Err(StepError::WaitForHoldRequest) => {
            master_thread_sender
                .send(MasterThreadMessage::Done(THREAD_ID))
                .unwrap();
            should_hold = true;
        }
        Err(StepError::BadMessage) => panic!("{THREAD_NAME} thread: bad message"),
    };
    while let Some((transition, episode_score)) = env.pop_transition() {
        if let Some(score) = episode_score {
            if let Some(ref mut plot_remote) = plot_remote {
                plot_remote.send(score);
            }
        }
        let mut agent = agent.write().unwrap();
        agent.remember(transition);
    }
    if schedule.is_time_to_update_params() {
        param_updater_thread_sender
            .send(ParamUpdaterThreadMessage::UpdateParams)
            .unwrap();
    }
    schedule.step();
    should_hold
}

fn wait_for_hold_message(receiver: &Receiver<EnvThreadMessage>) {
    loop {
        if matches!(
            receiver.recv().unwrap(),
            EnvThreadMessage::Master(MasterMessage::Hold)
        ) {
            return;
        }
    }
}

fn communicate_hold_sequence(
    receiver: &Receiver<EnvThreadMessage>,
    master_thread_sender: &Sender<MasterThreadMessage>,
) {
    match receiver.recv().unwrap() {
        EnvThreadMessage::Master(MasterMessage::PrepareHold) => {}
        _ => panic!("{THREAD_NAME} thread: bad message"),
    }
    master_thread_sender
        .send(MasterThreadMessage::Done(THREAD_ID))
        .unwrap();
    wait_for_hold_message(receiver);
}

pub enum EnvThreadMessage {
    Frame((ImageOwned2, u32)),
    Master(MasterMessage),
    WaitForHold,
}

enum ThreadMode {
    Running(ConcreteEnv),
    Held,
}

pub struct EnvThread {}

impl ThreadType for EnvThread {
    type Message = EnvThreadMessage;
    type SpawnArgs = (
        Sender<MasterThreadMessage>,
        Sender<UiThreadMessage>,
        Sender<GameThreadMessage>,
        ActorSettings,
    );

    fn spawn(receiver: Receiver<Self::Message>, args: Self::SpawnArgs) -> JoinHandle<()> {
        std::thread::spawn(move || {
            let (master_thread_sender, ui_thread_sender, game_thread_sender, settings) = args;
            const PARAM_UPDATE_INTERVAL_STEPS: u32 = 400;
            const ALPHA: f64 = 0.6;
            let agent =
                RemoteReplayWrapper::wrap(BasicModel::new(), settings.replay_server_addr, ALPHA);
            let agent = Arc::new(RwLock::new(agent));
            let (param_updater_thread_sender, param_updater_thread_receiver) =
                crossbeam_channel::unbounded::<ParamUpdaterThreadMessage>();
            let param_updater_thread = spawn_param_updater_thread(
                param_updater_thread_receiver,
                Arc::clone(&agent),
                &settings,
            );
            let mut schedule = ActorSchedule::new(settings.eps, PARAM_UPDATE_INTERVAL_STEPS);
            let mut plot_remote = settings
                .plot_server_addr
                .map(|addr| ActorPlotRemote::new(addr, settings.id, 10));
            let mut mode = ThreadMode::Held;
            loop {
                match mode {
                    ThreadMode::Held => match receiver.recv().unwrap() {
                        EnvThreadMessage::Master(message) => match message {
                            MasterMessage::Save(path) => {
                                schedule.save(path.as_path());
                                {
                                    let agent = agent.read().unwrap();
                                    agent.save(path);
                                }
                                master_thread_sender
                                    .send(MasterThreadMessage::Done(THREAD_ID))
                                    .unwrap();
                            }
                            MasterMessage::Load(path) => {
                                schedule.load(path.as_path());
                                {
                                    let mut agent = agent.write().unwrap();
                                    agent.load(path);
                                }
                                master_thread_sender
                                    .send(MasterThreadMessage::Done(THREAD_ID))
                                    .unwrap();
                            }
                            message @ (MasterMessage::Hold | MasterMessage::PrepareHold) => {
                                eprintln!("{THREAD_NAME} thread: {:?} while already held", message);
                            }
                            MasterMessage::Resume => {
                                match Env::new(receiver.clone(), game_thread_sender.clone()) {
                                    Ok(env) => {
                                        mode = ThreadMode::Running(env);
                                    }
                                    Err(StepError::WaitForHoldRequest) => {
                                        master_thread_sender
                                            .send(MasterThreadMessage::Done(THREAD_ID))
                                            .unwrap();
                                        communicate_hold_sequence(&receiver, &master_thread_sender);
                                    }
                                    Err(StepError::BadMessage) => {
                                        panic!("{THREAD_NAME} thread: bad message")
                                    }
                                };
                            }
                            MasterMessage::Close => {
                                break;
                            }
                        },
                        _ => panic!("{THREAD_NAME} thread: bad message"),
                    },
                    ThreadMode::Running(ref mut env) => {
                        let should_hold = step(
                            env,
                            &agent,
                            &mut schedule,
                            &master_thread_sender,
                            &ui_thread_sender,
                            &mut plot_remote,
                            &param_updater_thread_sender,
                        );
                        if should_hold {
                            mode = ThreadMode::Held;
                            communicate_hold_sequence(&receiver, &master_thread_sender);
                        }
                    }
                }
            }
            param_updater_thread_sender
                .send(ParamUpdaterThreadMessage::Stop)
                .unwrap();
            param_updater_thread.join().unwrap();
        })
    }

    fn master_message(msg: MasterMessage) -> Self::Message {
        Self::Message::Master(msg)
    }
}
