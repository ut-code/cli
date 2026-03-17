# coding-human

> **[日本語版はこちら](docs/README.ja.md)**

A real-time Q&A tool that connects a **client** (someone with a question) to a **coder** (someone who can answer) over WebSocket — entirely in the terminal, with a full TUI powered by [ratatui](https://ratatui.rs).

---

## Install

```sh
cargo install --git https://github.com/ut-code/coding-human --bin coding-human
```

Or build from source:

```sh
git clone https://github.com/ut-code/coding-human
cd coding-human
cargo build --release
# binary is at ./target/release/coding-human
```

## Quick start

You need two terminals — one for the **coder**, one for the **client**.

```sh
# Terminal 1 — become a coder and wait for questions
coding-human coder "Alice (Rust / systems)"

# Terminal 2 — connect as a client
coding-human client "Bob"
```

The client picks a coder from an interactive list, then a split-screen chat opens for both sides.

---

## Client guide

### Connect

```sh
coding-human client "Your Name"
```

An interactive picker appears listing all available coders. Use **↑ / ↓** (or **j / k**) to navigate and **Enter** to connect.

### Ask a question

Type your question in the input box at the bottom and press **Enter**.

```
What does the lifetime 'a mean here?
```

The input box is **locked** while you wait for the coder's answer — a "Waiting for answer…" indicator appears. When the answer arrives it streams into the chat log in real time.

### Attach a file

Prefix any path with `@` to send its contents to the coder before your question:

```
@src/main.rs why is the borrow checker rejecting this?
```

The file is read from disk and sent automatically. The coder receives it at `/tmp/coding-human/src/main.rs` and can open it directly.

### Respond to a command request

The coder may ask you to run a shell command. When that happens, you will see:

```
ⓘ  Run: ls -la?
ⓘ  [y/n]: type y or n and press Enter
```

Type **y** (execute) or **n** (skip) and press **Enter**. The output is sent back to the coder automatically.

### Respond to a diff

The coder may send a proposed file change. You will see the unified diff in the log:

```
ⓘ  Diff for src/main.rs:
   --- src/main.rs
   +++ src/main.rs
   @@ -10,3 +10,5 @@
   ...
ⓘ  apply? [y/n]: type y or n and press Enter
```

Type **y** to apply the patch or **n** to reject it.

### Auto-approve with `--yes`

Pass `--yes` to automatically execute all commands and apply all diffs without prompting:

```sh
coding-human client "Bob" --yes
```

### Keyboard shortcuts

| Key | Action |
|-----|--------|
| **Enter** | Send message |
| **Ctrl+C** | Quit |
| **↑ / ↓** | Scroll the log one line |
| **PgUp / PgDn** | Scroll the log half a page |

---

## Coder guide

### Join the queue

```sh
coding-human coder "Alice (Rust / systems)"
```

Your name appears in the client's picker. The input is locked until a client connects.

### Answer questions

Once a client connects, their question appears in the chat log and the input box unlocks. Type your answer line by line. Each line is sent to the client as you press **Enter**.

### Finish your answer

Press **Ctrl+D** when you are done with a response. This signals the client that the answer is complete and locks your input until the next question arrives.

### Run a command on the client's machine

Prefix a line with `$ ` (dollar + space) to ask the client to run a shell command. The client sees a y/n prompt; if they approve, the output is shown back to you automatically.

```
$ ls -la
```

```
$ cat package.json
```

### Open a client's file in your editor

If the client attached a file with `@path`, you can open it in your `$EDITOR` by typing `@` followed by the path (relative to `/tmp/coding-human/`). The TUI suspends cleanly, your editor opens, and the TUI resumes when you quit the editor.

```
@src/main.rs
```

### Send a diff back to the client

After editing a file, generate a unified diff and send it to the client for review:

```
$send-diff src/main.rs
```

The client sees the diff and is prompted to apply or reject it.

### Keyboard shortcuts

| Key | Action |
|-----|--------|
| **Enter** | Send a line of your answer |
| **Ctrl+D** | Finish current answer (sends `Done`) |
| **Ctrl+C** | Quit |
| **↑ / ↓** | Scroll the log |
| **PgUp / PgDn** | Scroll the log half a page |

---

## Configuration

The server URL defaults to `http://localhost:8787`. Override it with an environment variable or a `.env` file:

```sh
export SERVER_URL=https://your-worker.workers.dev
```

```sh
# .env
SERVER_URL=https://your-worker.workers.dev
```

---

## See also

- [Developer guide (English)](docs/development.md)
- [Developer guide (日本語)](docs/development.ja.md)
- [User guide (日本語)](docs/README.ja.md)
