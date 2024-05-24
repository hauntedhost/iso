use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Serialize, Deserialize, Debug)]
pub struct Room {
    pub name: String,
    pub player_count: u32,
}
