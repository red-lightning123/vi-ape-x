pub enum ThreadId {
    Game,
    Ui,
    Plot,
    Env,
}

impl ThreadId {
    pub fn as_bit_flag(&self) -> u8 {
        match self {
            Self::Game => 1,
            Self::Ui => 2,
            Self::Plot => 4,
            Self::Env => 8,
        }
    }
    pub fn all_flags() -> u8 {
        Self::Game.as_bit_flag()
            | Self::Ui.as_bit_flag()
            | Self::Plot.as_bit_flag()
            | Self::Env.as_bit_flag()
    }
}
