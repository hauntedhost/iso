use super::{message::Message as SocketMessage, refs::Refs, user::User};
use bevy::prelude::*;
use serde_json::{json, Value as SerdeValue};

// This module contains the Request struct used to create requests to be sent to the server.

const TOPIC_PREFIX: &str = "relay:";

// TODO: maybe Request should be an enum, e.g. Heartbeat, Join, Leave, Shout
#[derive(Clone, Debug)]
pub struct Request {
    pub event: String,
    payload: SerdeValue,
    topic: String,
}

impl Request {
    #[allow(dead_code)]
    pub fn new_heartbeat() -> Self {
        Self {
            topic: "phoenix".to_string(),
            event: "heartbeat".to_string(),
            payload: json!({}),
        }
    }

    pub fn new_join(room: String, user: User) -> Self {
        Self {
            topic: room_to_topic(room),
            event: "phx_join".to_string(),
            payload: json!({ "user": user  }),
        }
    }

    #[allow(dead_code)]
    pub fn new_leave(room: String) -> Self {
        Self {
            topic: room_to_topic(room),
            event: "phx_leave".to_string(),
            payload: json!({}),
        }
    }

    #[allow(dead_code)]
    pub fn new_shout(room: String, message: String) -> Self {
        Self {
            topic: room_to_topic(room),
            event: "shout".to_string(),
            payload: json!({  "message": message }),
        }
    }

    pub fn new_shout_location(room: String, position: Vec3) -> Self {
        let message = format!("{}, {}", position.x, position.z);
        Self {
            topic: room_to_topic(room),
            event: "shout".to_string(),
            payload: json!({  "message": message, "position": position }),
        }
    }

    pub fn to_payload(&self, refs: Refs) -> String {
        let message = SocketMessage {
            join_ref: Some(refs.get_join_ref()),
            message_ref: Some(refs.get_message_ref()),
            topic: self.topic.clone(),
            event: self.event.clone(),
            payload: self.payload.clone(),
        };

        message
            .serialize_to_json_string()
            .expect("Problem serializing message")
    }
}

fn room_to_topic(room: String) -> String {
    format!("{TOPIC_PREFIX}{room}")
}
