use super::player::Player;
use bevy::{prelude::*, utils::HashMap};
use chrono::Utc;

#[derive(Debug, Resource, Reflect)]
#[reflect(Resource)]
pub struct PlayerStore {
    pub player_uuid: String,
    pub players: HashMap<String, Player>,
}

impl Default for PlayerStore {
    fn default() -> Self {
        let player = Player::new_with_username_from_env_or_generate();
        let mut players = HashMap::new();
        players.insert(player.uuid.clone(), player.clone());

        Self {
            player_uuid: player.uuid,
            players,
        }
    }
}

impl PlayerStore {
    pub fn get_player(&self) -> &Player {
        self.players.get(&self.player_uuid).unwrap()
    }

    pub fn get_friend(&self, player_uuid: &String) -> Option<Player> {
        match self.get_friends().get(player_uuid) {
            Some(friend) => Some(friend.clone()),
            None => None,
        }
    }

    pub fn get_friends(&self) -> HashMap<String, Player> {
        let mut friends = self.players.clone();
        friends.remove(&self.player_uuid);
        friends
    }

    pub fn is_player_self(&self, player_uuid: &String) -> bool {
        &self.player_uuid == player_uuid
    }

    pub fn remove_friend(&mut self, player: Player) {
        if !self.is_player_self(&player.uuid) {
            self.players.remove(&player.uuid);
        }
    }

    pub fn upsert_player(&mut self, player: Player) {
        self.players
            .entry(player.uuid.clone())
            .and_modify(|existing_player| {
                if player.position.is_some() {
                    existing_player.position = player.position;
                }
            })
            .or_insert_with(|| player.clone());
    }

    pub fn upsert_players(&mut self, players: Vec<Player>) {
        for player in players {
            self.upsert_player(player);
        }
    }

    pub fn update_player_position(&mut self, player_uuid: String, position: Vec3) {
        if let Some(player) = self.players.get_mut(&player_uuid) {
            player.position = Some(position);
        }
    }

    pub fn has_spawned(&mut self, player_uuid: &String) -> bool {
        if let Some(player) = self.players.get(player_uuid) {
            player.spawned_at.is_some()
        } else {
            false
        }
    }

    pub fn set_spawned_at(&mut self, player_uuid: &String) {
        if let Some(player) = self.players.get_mut(player_uuid) {
            let now = Utc::now();
            let timestamp = now.timestamp() as u64;
            player.spawned_at = Some(timestamp);
        }
    }
}
