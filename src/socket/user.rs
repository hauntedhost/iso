use super::names::{generate_uuid, generate_valid_username};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Default, Clone, Serialize, Deserialize, Debug, Resource)]
pub struct User {
    pub uuid: String,
    pub username: String,
    pub online_at: u64,
    pub position: Option<Vec3>,
}

impl User {
    pub fn new(username: String) -> Self {
        Self {
            uuid: generate_uuid(),
            username,
            ..Default::default()
        }
    }

    // Create a new user from NAME env var otherwise generate a guest username
    pub fn new_from_env_or_generate() -> Self {
        match env::var("NAME") {
            Ok(username) => Self::new(username),
            Err(_) => Self::new(generate_valid_username()),
        }
    }

    #[allow(dead_code)]
    // Display name is username plus first four characters of uuid
    pub fn display_name(&self) -> String {
        let uuid_bit = self.uuid[0..4].to_string();
        format!("{}#{}", self.username, uuid_bit)
    }
}
