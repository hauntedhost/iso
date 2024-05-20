use serde_json::{Result as SerdeResult, Value as SerdeValue};

/// This module contains the `Message` struct with implementation logic for:
///   - Parsing JSON from the server into the `Message` struct
///   - Serializing the `Message` struct into a JSON payload

// The server sends and receives messages as a 5-element JSON array:
type MessageArray = (
    Option<String>, // join_ref
    Option<usize>,  // message_ref
    String,         // topic
    String,         // event
    SerdeValue,     // payload
);

type ResponseMessage = MessageArray;
type RequestMessage = MessageArray;

#[derive(Default, Debug)]
pub struct Message {
    pub join_ref: Option<String>,
    pub message_ref: Option<usize>,
    pub topic: String,
    pub event: String,
    pub payload: SerdeValue,
}

impl Message {
    // Parse server payload into Message struct
    pub fn new_from_json_string(json_data: &str) -> SerdeResult<Self> {
        let message_array: ResponseMessage = serde_json::from_str(json_data)?;
        let message = Self {
            join_ref: message_array.0,
            message_ref: message_array.1,
            topic: message_array.2,
            event: message_array.3,
            payload: message_array.4,
        };

        Ok(message)
    }

    // Serialize Message struct into JSON payload
    pub fn serialize_to_json_string(&self) -> SerdeResult<String> {
        let message_array: RequestMessage = (
            self.join_ref.clone(),
            self.message_ref,
            self.topic.clone(),
            self.event.clone(),
            self.payload.clone(),
        );
        let json = serde_json::to_string(&message_array)?;
        Ok(json)
    }
}
