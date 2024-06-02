mod actor_schedule;
mod env;
mod plot_datum_sender;
mod state_accums;

use crate::{
    GameThreadMessage, MasterMessage, MasterThreadMessage, PlotThreadMessage, ThreadId,
    UiThreadMessage,
};
use actor_schedule::ActorSchedule;
use crossbeam_channel::{Receiver, Sender};
use env::{Env, StepError};
use image::ImageOwned2;
use model::traits::{Actor, ParamFetcher, Persistable, TargetNet};
use model::{BasicModel, Params};
use packets::{GetParamsReply, LearnerRequest};
use plot_datum_sender::PlotDatumSender;
use rand::Rng;
use replay_data::State;
use replay_wrappers::RemoteReplayWrapper;
use state_accums::filters::{CompressFilter, Filter};
use state_accums::{FrameStack, PipeFilterToAccum};
use std::net::TcpStream;

type Accum = PipeFilterToAccum<CompressFilter, FrameStack<<CompressFilter as Filter>::Output>>;
type ConcreteEnv = Env<Accum>;

fn random_action() -> u8 {
    rand::thread_rng().gen_range(0..ConcreteEnv::n_actions())
}

const THREAD_ID: ThreadId = ThreadId::Env;
const THREAD_NAME: &str = "env";

fn get_params_from_learner() -> Params {
    let request = LearnerRequest::GetParams;
    let stream = match TcpStream::connect(("localhost", ports::LEARNER)) {
        Ok(stream) => stream,
        Err(e) => {
            panic!("Could not connect to learner: {}", e);
        }
    };
    tcp_io::serialize_into(&stream, &request).unwrap();
    let GetParamsReply { params } = tcp_io::deserialize_from(stream).unwrap();
    params
}

fn step(
    env: &mut ConcreteEnv,
    agent: &mut RemoteReplayWrapper<BasicModel>,
    schedule: &mut ActorSchedule,
    master_thread_sender: &Sender<MasterThreadMessage>,
    ui_thread_sender: &Sender<UiThreadMessage>,
    plot_datum_sender: &PlotDatumSender,
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
            plot_datum_sender.send_episode_score(score, schedule);
        }
        agent.remember(transition);
    }
    if schedule.is_time_to_update_params() {
        let params = get_params_from_learner();
        agent.set_params(params);
    }
    schedule.step();
    should_hold
}

fn wait_for_hold_message(receiver: &Receiver<EnvThreadMessage>) {
    loop {
        if let EnvThreadMessage::Master(MasterMessage::Hold) = receiver.recv().unwrap() {
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

pub fn spawn_env_thread(
    receiver: Receiver<EnvThreadMessage>,
    master_thread_sender: Sender<MasterThreadMessage>,
    ui_thread_sender: Sender<UiThreadMessage>,
    game_thread_sender: Sender<GameThreadMessage>,
    plot_thread_sender: Sender<PlotThreadMessage>,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        const PARAM_UPDATE_INTERVAL_STEPS: u32 = 400;
        const ALPHA: f64 = 0.6;
        let plot_datum_sender = PlotDatumSender::new(plot_thread_sender);
        let eps = rand::thread_rng().gen();
        let mut schedule = ActorSchedule::new(eps, PARAM_UPDATE_INTERVAL_STEPS);
        let mut agent = RemoteReplayWrapper::wrap(BasicModel::new(), ALPHA);
        let mut mode = ThreadMode::Held;
        loop {
            match mode {
                ThreadMode::Held => match receiver.recv().unwrap() {
                    EnvThreadMessage::Master(message) => match message {
                        MasterMessage::Save(path) => {
                            schedule.save(path.as_path());
                            agent.save(path);
                            master_thread_sender
                                .send(MasterThreadMessage::Done(THREAD_ID))
                                .unwrap();
                        }
                        MasterMessage::Load(path) => {
                            schedule.load(path.as_path());
                            agent.load(path);
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
                        &mut agent,
                        &mut schedule,
                        &master_thread_sender,
                        &ui_thread_sender,
                        &plot_datum_sender,
                    );
                    if should_hold {
                        mode = ThreadMode::Held;
                        communicate_hold_sequence(&receiver, &master_thread_sender);
                    }
                }
            }
        }
    })
}
