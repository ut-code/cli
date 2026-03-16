use serde::{Deserialize, Serialize};

/// Messages exchanged over the WebSocket connection.
/// The worker relays these as opaque text; both sides parse them with this type.
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WsMessage {
    /// Programmer → Client: run a shell command
    Cmd { command: String },
    /// Client → Programmer: result of a shell command
    CmdResult { command: String, output: String },
    /// Client → Programmer: contents of a file referenced with @filepath
    File { path: String, content: String },
    /// Programmer → Client: unified diff of an edited file
    Diff { path: String, diff: String },
    /// Client → Programmer: response to a received diff (accepted or rejected)
    DiffResponse { accepted: bool },
}
