#![allow(dead_code)]
mod image;
mod game_interface;
mod human_interface;
mod keycodes;
mod x11_utils;
mod game;
mod file_io;
use image::{ ImageOwned2, Color2, ImageRef4, Color4, ImageOwned, ImageRef };
use game_interface::{ GameInterface, GameKey, KeyEventKind, Window };
use human_interface::HumanInterface;
use x11_utils::{ X11Display, GlxContext };
use game::Game;

mod master_thread;
use master_thread::{ spawn_master_thread, MasterThreadMessage, MasterMessage, ThreadId, Query };
mod game_thread;
use game_thread::{ spawn_game_thread, GameThreadMessage };
mod ui_thread;
use ui_thread::{ spawn_ui_thread, UiThreadMessage };
mod env_thread;
use env_thread::{ spawn_env_thread, EnvThreadMessage };
mod plot_thread;
use plot_thread::{ spawn_plot_thread, PlotThreadMessage };

fn main() {
    let master_thread = spawn_master_thread();
    master_thread.join().unwrap();
}
