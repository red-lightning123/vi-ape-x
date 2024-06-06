pub enum ThreadId {
    Game,
    Ui,
    Env,
}

impl ThreadId {
    pub fn as_bit_flag(&self) -> u8 {
        match self {
            Self::Game => 1,
            Self::Ui => 2,
            Self::Env => 4,
        }
    }
    pub fn all_flags() -> u8 {
        Self::Game.as_bit_flag() | Self::Ui.as_bit_flag() | Self::Env.as_bit_flag()
    }
}
