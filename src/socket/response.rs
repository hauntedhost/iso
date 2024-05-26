use super::message::Message;
use crate::player::player::Player;
use bevy::math::Vec3;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// This module contains logic for parsing JSON from the server.
/// Response struct exposes a single `new_from_json_string` fn which takes a JSON string and returns a `Response` enum.

// The Response enum we will build based on the event type
#[derive(Clone, Default, Debug)]
pub enum Response {
    Ack(Ack),
    JoinReply(JoinReply),
    PlayerUpdate(PlayerUpdate),
    PresenceDiff(PresenceDiff),
    PresenceState(PresenceState),
    RoomsUpdate(RoomsUpdate),
    Shout(Shout),
    #[default]
    Unknown,
}

impl Response {
    pub fn new_from_json_string(json_data: &str) -> Self {
        let Ok(message) = Message::new_from_json_string(json_data) else {
            return Response::Unknown;
        };

        return match message.event.as_str() {
            "phx_reply" => {
                if message.topic == "phoenix" {
                    if let Ok(reply) = serde_json::from_value::<RawAckReply>(message.payload) {
                        return Response::Ack(Ack {
                            status: reply.status,
                        });
                    }
                } else {
                    if let Ok(reply) = serde_json::from_value::<RawJoinReply>(message.payload) {
                        if reply.response.event == "phx_join" {
                            return Response::JoinReply(JoinReply {
                                player: reply.response.player,
                            });
                        }
                    }
                }
                Response::Unknown
            }
            "player_update" => {
                let player_update =
                    serde_json::from_value::<PlayerUpdate>(message.payload).unwrap();
                Response::PlayerUpdate(player_update)
            }
            "presence_diff" => {
                let raw_diff = serde_json::from_value::<RawPresenceDiff>(message.payload).unwrap();
                let joins = get_players(raw_diff.joins);
                let leaves = get_players(raw_diff.leaves);
                Response::PresenceDiff(PresenceDiff { joins, leaves })
            }
            "presence_state" => {
                let raw_state =
                    serde_json::from_value::<RawPresenceState>(message.payload).unwrap();
                let players = get_players(raw_state);
                Response::PresenceState(PresenceState { players })
            }
            "rooms_update" => {
                let rooms_update =
                    serde_json::from_value::<RawRoomsUpdate>(message.payload).unwrap();
                let rooms: Vec<Room> = rooms_update
                    .rooms
                    .iter()
                    .map(|room_update| Room {
                        name: room_update.0.clone(),
                        player_count: room_update.1,
                    })
                    .collect();
                Response::RoomsUpdate(rooms)
            }
            "shout" => {
                let shout = serde_json::from_value::<Shout>(message.payload).unwrap();
                Response::Shout(shout)
            }
            _ => Response::Unknown,
        };
    }
}

#[derive(Clone, Default, Serialize, Deserialize, Debug)]
pub struct Ack {
    pub status: String,
}

#[derive(Clone, Default, Serialize, Deserialize, Debug)]
pub struct JoinReply {
    pub player: Player,
}

#[derive(Clone, Default, Serialize, Deserialize, Debug)]
pub struct PlayerUpdate {
    pub player_uuid: String,
    pub position: Vec3,
}

#[derive(Clone, Default, Debug)]
pub struct PresenceDiff {
    pub joins: Vec<Player>,
    pub leaves: Vec<Player>,
}

#[derive(Clone, Default, Serialize, Deserialize, Debug)]
pub struct PresenceState {
    pub players: Vec<Player>,
}

#[derive(Clone, Default, Serialize, Deserialize, Debug)]
pub struct Room {
    pub name: String,
    pub player_count: u32,
}

pub type RoomsUpdate = Vec<Room>;

// Private

#[derive(Clone, Default, Serialize, Deserialize, Debug)]
pub struct Shout {
    pub player: Player,
    pub message: String,
    pub position: Option<Vec3>,
}

#[derive(Default, Serialize, Deserialize, Debug)]
struct RawAckReply {
    status: String,
}

#[derive(Default, Serialize, Deserialize, Debug)]
struct RawJoinReply {
    status: String,
    response: RawJoinReplyResponse,
}

#[derive(Default, Serialize, Deserialize, Debug)]
struct RawJoinReplyResponse {
    event: String,
    player: Player,
}

#[derive(Default, Serialize, Deserialize, Debug)]
struct RawRoomsUpdate {
    rooms: Vec<RoomUpdateArray>,
}

type RoomUpdateArray = (
    String, // name
    u32,    // player count
);

#[derive(Default, Serialize, Deserialize, Debug)]
struct RawPresenceDiff {
    joins: HashMap<String, PlayerPresence>,
    leaves: HashMap<String, PlayerPresence>,
}

type RawPresenceState = HashMap<String, PlayerPresence>;

#[derive(Serialize, Deserialize, Debug)]
struct PlayerMeta {
    uuid: String,
    username: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct PlayerPresence {
    metas: Vec<PlayerMeta>,
    player: Player,
}

fn get_players(player_presences: HashMap<String, PlayerPresence>) -> Vec<Player> {
    let mut players = Vec::new();
    for (_key, player_presence) in player_presences {
        players.push(player_presence.player);
    }
    players
}
