# Development Guide

> **[日本語版](development.ja.md)**

This document explains the internal architecture, module layout, and contribution workflow for the `coding-human` project.

---

## Repository layout

```
coding-human/
├── Cargo.toml          # workspace root (members: cli, worker)
├── Cargo.lock
├── .env                # local overrides (SERVER_URL)
├── flake.nix           # Nix dev-shell (optional)
│
├── cli/                # Native binary — coding-human
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs     # CLI entry point (clap commands: client / coder)
│       ├── protocol.rs # Shared WebSocket message types (WsMessage enum)
│       ├── tui.rs      # Full ratatui TUI: picker, chat widget, input lock
│       ├── client.rs   # Client mode: pick coder, Q&A loop
│       └── coder.rs    # Coder mode: register, answer loop
│
└── worker/             # Cloudflare Worker — relay server (Rust → WASM)
    ├── Cargo.toml
    ├── wrangler.jsonc
    └── src/
        └── lib.rs      # QueueDO + RoomSession + fetch handler
```

---

## Local development

### Prerequisites

| Tool | Purpose |
|------|---------|
| `rustup` + stable toolchain | Build the CLI |
| `cargo` | Build / check / test |
| Node.js + `npx` | Run `wrangler dev` (worker) |
| `wrangler` (via npx) | Cloudflare Worker dev server |

### Start the worker

```sh
cd worker
npx wrangler dev          # listens on http://localhost:8787
```

### Run the CLI against the local worker

```sh
# Terminal 1 — coder
cargo run --bin coding-human -- coder "Alice"

# Terminal 2 — client
cargo run --bin coding-human -- client "Bob"
```

The `.env` file at the workspace root is loaded automatically via `dotenvy`:

```
SERVER_URL=http://localhost:8787
```

---

## Architecture

### WebSocket protocol (`protocol.rs`)

All messages are serialised as JSON with a `type` discriminator tag (`serde(tag = "type", rename_all = "snake_case")`):

| Variant | Direction | Purpose |
|---------|-----------|---------|
| `Matched { client_name }` | Client → Coder | Handshake on connect |
| `Question { from, text }` | Client → Coder | A question |
| `File { path, content }` | Client → Coder | File sent with `@path` |
| `Cmd { command }` | Coder → Client | Ask client to run a command |
| `CmdResult { command, output }` | Client → Coder | Result of executed command |
| `Diff { path, diff }` | Coder → Client | Unified diff to apply |
| `DiffResponse { accepted }` | Client → Coder | Whether diff was applied |
| `Done` | Coder → Client | End of current answer stream |

Any message that fails to parse as a `WsMessage` is treated as raw answer text.

### TUI (`tui.rs`)

Built with [ratatui](https://ratatui.rs) + [crossterm](https://docs.rs/crossterm). Key types:

#### `ChatMsg` (async task → TUI)

```rust
pub enum ChatMsg {
    Msg { role: ChatRole, text: String },
    SetWaiting(bool),      // lock / unlock the input box
    OpenEditor(String),    // suspend TUI, open $EDITOR, resume
}
```

#### `ChatEvent` (TUI → async task)

```rust
pub enum ChatEvent {
    Line(String),     // user pressed Enter
    Eof,              // Ctrl+D
    Quit,             // Ctrl+C
    EditorClosed,     // editor subprocess exited
}
```

#### Concurrency model

```
┌──────────────────────────────┐   log_tx (ChatMsg)   ┌──────────────────┐
│   net_task (tokio::spawn)    │ ──────────────────►  │  run_chat (main  │
│                              │                       │  async task)     │
│  WebSocket I/O               │ ◄──────────────────  │                  │
│  Command execution           │   event_tx (ChatEvent)│  Terminal draw   │
│  Patch application           │                       │  Key events      │
└──────────────────────────────┘                       └──────────────────┘
```

`run_chat` owns the `Terminal<CrosstermBackend<Stdout>>`. It pre-renders all chat entries into `Vec<Line<'static>>` each frame so that the scrollbar's position (in rendered-line units) is always consistent.

#### Scrolling

`scroll_offset` is tracked in **rendered-line units**, not message-entry units. `at_bottom = true` means the view follows the tail automatically; the effective offset is computed as `total_rendered_lines - inner_height` each frame.

#### Editor integration (coder only)

When the coder types `@src/main.rs`, the net task sends `ChatMsg::OpenEditor(path)` instead of spawning the editor directly. `run_chat` handles it by:

1. `leave(&mut term)` — exits alternate screen + raw mode
2. `std::process::Command::new(editor).arg(path).status()` — runs the editor in a clean terminal
3. Re-enters raw mode + alternate screen; calls `term.clear()`
4. Sends `ChatEvent::EditorClosed` to unblock the net task

This avoids the broken-layout bug that occurs when an external process writes to stdout while ratatui owns the alternate screen.

---

## Adding new message types

1. Add the variant to `WsMessage` in `protocol.rs`.
2. Handle it in `coder.rs` (send side) and `client.rs` (receive side), or vice versa.
3. Use `log_tx.send(ChatMsg::sys(...))` to show status in the TUI log.

---

## Deploying the worker

The Cloudflare Worker is compiled to WASM via [workers-rs](https://github.com/cloudflare/workers-rs):

```sh
# Install worker-build once
cargo install worker-build

cd worker
npx wrangler deploy
```

### Worker API

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/queue` | List waiting coders `{ roomId: label }` |
| `POST` | `/queue` | Register a coder `{ label }` → `{ roomId }` |
| `DELETE` | `/queue/:roomId` | Deregister a coder |
| `GET` (WS) | `/rooms/:id/coder` | WebSocket endpoint for the coder |
| `GET` (WS) | `/rooms/:id/client` | WebSocket endpoint for the client |

### Durable Objects

| Object | Instances | Purpose |
|--------|-----------|---------|
| `QueueDO` | Singleton | Persists the coder queue in KV storage |
| `RoomSession` | One per room | Relays messages between coder ↔ client via hibernatable WebSockets |

---

## Coding conventions

- **Error handling**: all fallible functions return `anyhow::Result`. Use `?` freely; propagate errors to `main` which prints them and exits with code 1.
- **Async**: `tokio` runtime with the `full` feature. Network I/O in a dedicated `tokio::spawn` task; TUI on the caller task.
- **Channels**: `mpsc::unbounded_channel` for TUI ↔ net-task communication. Bounded channels are unnecessary here because the TUI draw loop drains messages faster than they can be produced.
- **Format**: `rustfmt` default style. Run `cargo fmt` before committing.
- **Lint**: `cargo clippy -- -D warnings`.
