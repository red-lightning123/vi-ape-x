#![allow(dead_code, unused_imports)]
mod file_io;
mod game;
mod game_interface;
mod human_interface;
mod image;
mod keycodes;
mod x11_utils;
use game::Game;
use game_interface::{GameInterface, GameKey, KeyEventKind};
use human_interface::HumanInterface;
use image::{Color2, Color4, ImageOwned, ImageOwned2, ImageRef, ImageRef2, ImageRef4};
use x11_utils::{choose_matching_fbconfigs, GlxContext, Window, X11Display};

mod master_thread;
use master_thread::{spawn_master_thread, MasterMessage, MasterThreadMessage, Query, ThreadId};
mod game_thread;
use game_thread::{spawn_game_thread, GameThreadMessage};
mod ui_thread;
use ui_thread::{spawn_ui_thread, UiThreadMessage};
mod env_thread;
use env_thread::{spawn_env_thread, EnvThreadMessage};
mod plot_thread;
use plot_thread::{spawn_plot_thread, PlotThreadMessage};

fn main() {
    let master_thread = spawn_master_thread();
    master_thread.join().unwrap();
}
