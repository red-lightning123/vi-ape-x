mod score_reader;

use crate::ImageRef4;
use crate::{GameInterface, GameKey, KeyEventKind};
use score_reader::{ScoreReader, ScoreReaderError};

pub struct Game<'a> {
    interface: GameInterface<'a>,
    score: u32,
    score_reader: ScoreReader,
}

impl<'a> Game<'a> {
    pub fn new() -> Game<'a> {
        Game {
            interface: GameInterface::new(),
            score: 0,
            score_reader: ScoreReader::new(),
        }
    }
    pub fn start(&mut self) {
        self.interface.start();
    }
    pub fn end(&mut self) {
        self.interface.end();
    }
    pub fn interface(&self) -> &GameInterface {
        &self.interface
    }
    pub fn send(&mut self, key: GameKey, kind: KeyEventKind) {
        self.interface.send(key, kind);
    }
    pub fn next_frame(&mut self) {
        self.interface.next_frame();
        match self.score_reader.read_score(self.get_current_frame()) {
            Ok(score) => self.score = score,
            Err(ScoreReaderError::InvalidColor) => {
                eprintln!("score reader encountered invalid color")
            }
            Err(ScoreReaderError::InvalidDigitShape) => {
                eprintln!("score reader encountered invalid digit shape")
            }
        }
    }
    pub fn get_current_frame(&self) -> ImageRef4 {
        self.interface.get_current_frame()
    }
    pub fn get_current_score(&self) -> u32 {
        self.score
    }
    pub fn wait_vsync(&mut self) {
        self.interface.wait_vsync();
    }
    pub fn terminate(self) {
        self.interface.terminate();
    }
}
