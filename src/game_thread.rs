use crate::game_interface::{ GameKey, KeyEventKind };
use crate::{ MasterMessage, MasterThreadMessage, UiThreadMessage, EnvThreadMessage, ThreadId };
use crate::Game;
use crossbeam_channel::{ Sender, Receiver };
use crate::{ Color2, Color4 };
use crate::{ ImageOwned2, ImageRef4, ImageOwned, ImageRef };

enum ThreadMode {
    Running(std::time::Instant),
    Held
}

fn keep_rg(image : &ImageRef4) -> ImageOwned2 {
    let mut rg = ImageOwned2::zeroed(image.width(), image.height());
    for y in 0..image.height() {
        for x in 0..image.width() {
            let original_color = image.get_pixel_color(x, y);
            rg.set_pixel_color(x, y, Color2::new(original_color.0, original_color.1));
        }
    }
    rg
}

fn black_out_score_area(frame : &mut ImageOwned2) {
    frame.replace_area_color((93, 128), (2, 24), Color2::new(0, 0));
}

fn preprocess_frame(frame : &ImageRef4) -> ImageOwned2 {
    const INPUT_FRAME_WIDTH : u32 = 1920;
    const INPUT_FRAME_HEIGHT : u32 = 1080;
    const DOWNSCALE_FACTOR : u32 = 120 / 8;
    let mut frame = frame.downscale_by_average(INPUT_FRAME_WIDTH / DOWNSCALE_FACTOR, INPUT_FRAME_HEIGHT / DOWNSCALE_FACTOR);
    frame.map_color(|c| { Color4::new(c.2, c.1, c.0, c.3) });
    let mut frame = keep_rg(&frame.as_ref());
    black_out_score_area(&mut frame);
    frame
}

pub enum GameThreadMessage {
    Action(u8),
    Truncation,
    Master(MasterMessage)
}

const THREAD_ID : ThreadId = ThreadId::Game;
const THREAD_NAME : &str = "game";

fn wait_for_hold_message(receiver : &Receiver<GameThreadMessage>) {
    loop {
        if let GameThreadMessage::Master(MasterMessage::Hold) = receiver.recv().unwrap() {
            return
        }
    }
}

pub fn spawn_game_thread(receiver : Receiver<GameThreadMessage>, master_thread_sender : Sender<MasterThreadMessage>, ui_thread_sender : Sender<UiThreadMessage>, env_thread_sender : Sender<EnvThreadMessage>) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        let mut mode = ThreadMode::Held;
        let mut game = Game::new();
        loop {
            match mode {
                ThreadMode::Held => {
                    match receiver.recv().unwrap() {
                        GameThreadMessage::Master(message) => {
                            match message {
                                MasterMessage::Save(_) => {
                                    master_thread_sender.send(MasterThreadMessage::Done(THREAD_ID)).unwrap();
                                }
                                MasterMessage::Load(_) => {
                                    master_thread_sender.send(MasterThreadMessage::Done(THREAD_ID)).unwrap();
                                }
                                message @ (MasterMessage::Hold | MasterMessage::PrepareHold) => {
                                    eprintln!("{THREAD_NAME} thread: {:?} while already held", message);
                                }
                                MasterMessage::Resume => {
                                    game.start();
                                    let win = game.interface().win().clone();
                                    ui_thread_sender.send(UiThreadMessage::WinDims(win)).unwrap();
                                    let current_start_time = std::time::Instant::now();
                                    mode = ThreadMode::Running(current_start_time);
                                }
                                MasterMessage::Close => {
                                    break;
                                }
                            }
                        }
                        _ => panic!("{THREAD_NAME} thread: bad message")
                    }
                }
                ThreadMode::Running(ref mut current_start_time) => {
                    game.next_frame();
                    let preprocessed_frame = preprocess_frame(&game.get_current_frame());
                    let score = game.get_current_score();
                    env_thread_sender.send(EnvThreadMessage::Frame((preprocessed_frame, score))).unwrap();

                    match receiver.recv().unwrap() {
                        GameThreadMessage::Action(action) => {
                            send_action_events(action, &mut game);
                            let next_start_time = *current_start_time + std::time::Duration::from_millis(100);
                            let now = std::time::Instant::now();
                            std::thread::sleep(next_start_time - now);
                        }
                        GameThreadMessage::Truncation => {
                            game.end();
                            game.start();
                        }
                        GameThreadMessage::Master(message) => {
                            match message {
                                MasterMessage::PrepareHold => {
                                    game.end();
                                    mode = ThreadMode::Held;
                                    master_thread_sender.send(MasterThreadMessage::Done(THREAD_ID)).unwrap();
                                    wait_for_hold_message(&receiver);
                                    continue;
                                }
                                _ => panic!("{THREAD_NAME} thread: bad message")
                            }
                        }
                    };
                    *current_start_time = std::time::Instant::now();
                }
            }
        }
        game.terminate();
    })
}

fn send_action_events(action : u8, game : &mut Game) {
    match action {
        0 => { },
        1 => press_and_release(GameKey::S, game),
        2 => press_and_release(GameKey::Space, game),
        _ => panic!("{THREAD_NAME} thread: action not in valid range")
    }
}

fn press_and_release(key : GameKey, game : &mut Game) {
    game.send(key, KeyEventKind::Press);
    wait_until_key_event_is_processed(game);
    game.send(key, KeyEventKind::Release);
    wait_until_key_event_is_processed(game);
}

fn wait_until_key_event_is_processed(game : &mut Game) {
    // TODO: this is a hack.
    // there may be a better way to ensure that the
    // event has been processed than simply waiting
    // for a constant safety factor.
    // also, the call to game.wait_vsync() might be
    // superfluous
    const WAIT_TIME : std::time::Duration = std::time::Duration::from_millis(20);
    game.wait_vsync();
    std::thread::sleep(WAIT_TIME);
}
