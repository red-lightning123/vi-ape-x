mod training_schedule;
use training_schedule::TrainingSchedule;
mod agent;
use agent::traits::{Actor, Persistable, TargetNet};
use agent::{BasicModel, PrioritizedReplayWrapper};
mod env;
use env::{Env, StepError};
mod plot_datum_sender;
use crate::{
    GameThreadMessage, MasterMessage, MasterThreadMessage, PlotThreadMessage, Query, ThreadId,
    UiThreadMessage,
};
use crate::{ImageOwned, ImageOwned2, ImageRef};
use crossbeam_channel::{Receiver, Sender};
use plot_datum_sender::PlotDatumSender;
use rand::Rng;
use std::rc::Rc;

type State = [Rc<ImageOwned2>; 4];
struct Transition {
    state: State,
    next_state: State,
    action: u8,
    reward: f64,
    terminated: bool,
}

fn state_dims(state: &State) -> (u32, u32) {
    let frame = &state[0];
    (frame.width(), frame.height())
}

fn concat_state_frames(state: &State) -> ImageOwned2 {
    let (width, height) = state_dims(state);
    let mut concated_image = ImageOwned2::new(
        width,
        height * (state.len() as u32),
        vec![0; 4 * (width * height * (state.len() as u32)) as usize],
    );
    for (n_frame, frame) in state.iter().enumerate() {
        for y in 0..height {
            for x in 0..width {
                concated_image.set_pixel_color(
                    x,
                    y + (n_frame as u32) * height,
                    frame.get_pixel_color(x, y),
                );
            }
        }
    }
    concated_image
}

fn random_action() -> u8 {
    rand::thread_rng().gen_range(0..Env::n_actions())
}

const THREAD_ID: ThreadId = ThreadId::Env;
const THREAD_NAME: &str = "env";

fn step(
    env: &mut Env,
    agent: &mut PrioritizedReplayWrapper<BasicModel>,
    schedule: &mut TrainingSchedule,
    master_thread_sender: &Sender<MasterThreadMessage>,
    ui_thread_sender: &Sender<UiThreadMessage>,
    plot_datum_sender: &PlotDatumSender,
) -> bool {
    let state = env.state();
    let concated_state = concat_state_frames(&state);
    ui_thread_sender
        .send(UiThreadMessage::Frame(concated_state))
        .unwrap();
    ui_thread_sender
        .send(UiThreadMessage::NStep(schedule.n_step()))
        .unwrap();
    let action = if schedule.is_on_eps_random() || rand::thread_rng().gen::<f64>() < schedule.eps()
    {
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
    if !schedule.is_on_eps_random() {
        const BETA_START: f64 = 0.4;
        const BETA_END: f64 = 1.0;
        const N_BETA_ANNEALING_FRAMES: u32 = 2_000_000;
        let beta = BETA_START
            + (BETA_END - BETA_START) * f64::from(schedule.n_step())
                / f64::from(N_BETA_ANNEALING_FRAMES);
        let beta = if beta > BETA_END { BETA_END } else { beta };
        if let Some(learning_step) = agent.train_step(beta) {
            plot_datum_sender.send_loss(learning_step.loss, schedule);
            plot_datum_sender.send_q_val(learning_step.average_q_val, schedule);
        }
        if schedule.is_time_to_update_target() {
            agent.copy_control_to_target();
        }
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
    Running(Env),
    Held,
}

pub fn spawn_env_thread(
    receiver: Receiver<EnvThreadMessage>,
    query_receiver: Receiver<Query>,
    master_thread_sender: Sender<MasterThreadMessage>,
    ui_thread_sender: Sender<UiThreadMessage>,
    game_thread_sender: Sender<GameThreadMessage>,
    plot_thread_sender: Sender<PlotThreadMessage>,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        const EPS_MIN: f64 = 0.1;
        const EPS_MAX: f64 = 1.0;
        const N_EPS_RANDOM_STEPS: u32 = 100_000;
        const N_EPS_GREEDY_STEPS: u32 = 1_000_000;
        const TARGET_UPDATE_INTERVAL_STEPS: u32 = 10_000;
        const MEMORY_CAPACITY: usize = 1_000_000;
        let plot_datum_sender = PlotDatumSender::new(plot_thread_sender);
        let mut schedule = TrainingSchedule::new(
            EPS_MIN,
            EPS_MAX,
            N_EPS_RANDOM_STEPS,
            N_EPS_GREEDY_STEPS,
            TARGET_UPDATE_INTERVAL_STEPS,
        );
        const ALPHA: f64 = 0.6;
        let mut agent = PrioritizedReplayWrapper::wrap(BasicModel::new(), MEMORY_CAPACITY, ALPHA);
        let mut mode = ThreadMode::Held;
        loop {
            if let Ok(query) = query_receiver.try_recv() {
                match query {
                    Query::NStep => {
                        master_thread_sender
                            .send(MasterThreadMessage::QueryReply(
                                schedule.n_step().to_string(),
                            ))
                            .unwrap();
                    }
                }
            }
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
