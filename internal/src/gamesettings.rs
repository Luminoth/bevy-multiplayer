#[derive(Debug)]
pub struct GameSettings {
    pub max_players: u16,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self { max_players: 3 }
    }
}
