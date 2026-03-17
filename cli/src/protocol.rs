use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// POST /queue request body — sent by the programmer when registering.
#[derive(Serialize, Deserialize, Debug)]
pub struct RegisterRequest {
    pub label: String,
}

/// POST /queue response body — returned by the worker after registering.
#[derive(Serialize, Deserialize, Debug)]
pub struct RegisterResponse {
    #[serde(rename = "roomId")]
    pub room_id: String,
}

/// GET /queue response body — map of room_id → label for all waiting programmers.
pub type QueueResponse = HashMap<String, String>;

/// Messages exchanged over the WebSocket connection.
/// The worker relays these as opaque text; both sides parse them with this type.
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WsMessage {
    /// Client → Programmer: sent immediately on connect to identify the client
    Matched { client_name: String },
    /// Client → Programmer: a question with the client's display name
    Question { from: String, text: String },
    /// Programmer → Client: run a shell command
    Cmd { command: String },
    /// Client → Programmer: result of a shell command
    CmdResult { command: String, output: String },
    /// Client → Programmer: contents of a file referenced with @filepath
    File { path: String, content: String },
    /// Programmer → Client: unified diff of an edited file
    Diff { path: String, diff: String },
    /// Programmer → Client: signals end of the current answer stream
    Done,
}
