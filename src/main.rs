#![allow(dead_code, unused_imports)]
mod env_thread;
mod file_io;
mod game;
mod game_interface;
mod game_thread;
mod human_interface;
mod image;
mod keycodes;
mod master_thread;
mod plot_thread;
mod ui_thread;
mod x11_utils;

use env_thread::{spawn_env_thread, EnvThreadMessage};
use game::Game;
use game_interface::{GameInterface, GameKey, KeyEventKind};
use game_thread::{spawn_game_thread, GameThreadMessage};
use human_interface::HumanInterface;
use image::{Color2, Color4, ImageOwned, ImageOwned2, ImageRef, ImageRef2, ImageRef4};
use master_thread::{spawn_master_thread, MasterMessage, MasterThreadMessage, ThreadId};
use plot_thread::{spawn_plot_thread, PlotThreadMessage, PlotType};
use ui_thread::{spawn_ui_thread, UiThreadMessage};
use x11_utils::{choose_matching_fbconfigs, GlxContext, Window, X11Display};

fn main() {
    let master_thread = spawn_master_thread();
    master_thread.join().unwrap();
}
