# coding-human


A CLI tool for real-time Q&A between a client and a programmer over WebSocket.

Programmers register themselves in a queue; clients browse the queue and connect to a programmer of their choice.

## Repository layout

```
coding-human/
├── Cargo.toml      # workspace root
├── cli/            # native binary (coding-human)
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs
│       ├── ask.rs   # client: browse queue, pick a programmer, ask questions
│       └── serve.rs # programmer: join queue, answer questions
└── worker/         # Cloudflare Worker (workers-rs / WASM)
    ├── Cargo.toml
    ├── wrangler.jsonc
    └── src/
        └── lib.rs   # QueueDO + RoomSession + fetch handler
```

## Install (CLI)

```sh
cargo install --git https://github.com/ut-code/cli --bin coding-human
```

## Usage

### Programmer: join the queue

```sh
coding-human serve <LABEL>
```

Registers you in the queue with the given display name, creates a room, and
waits for a client to connect. Once connected, questions arrive automatically
and you type answers line by line. Press `Ctrl+D` to finish a response and
wait for the next question.

```sh
coding-human serve "Alice (Rust / systems)"
```

### Client: ask a question

```sh
coding-human ask
```

Fetches the list of available programmers, lets you pick one, then opens a
session. Type questions at the prompt; answers stream back in real time.
Type `/quit` or press `Ctrl+D` to exit.

## Configuration (CLI)

The server URL defaults to `http://localhost:8787`. Override with a
`SERVER_URL` environment variable or a `.env` file:

```
SERVER_URL=https://your-worker.example.com
```

## Local development

```sh
# install worker-build once
cargo install worker-build

# start the worker locally on http://localhost:8787
cd worker
npx wrangler dev
```

Then run the CLI against it (default URL is already `http://localhost:8787`):

```sh
# terminal 1 — programmer
coding-human serve "Alice"

# terminal 2 — client
coding-human ask
```

## Deploy (Worker)

The Cloudflare Worker is written in Rust using
[workers-rs](https://github.com/cloudflare/workers-rs) and compiled to WASM.

```sh
# install worker-build once
cargo install worker-build

# build + deploy
cd worker
npx wrangler deploy
```

The worker exposes:

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/queue` | List waiting programmers `{ roomId: label }` |
| `POST` | `/queue` | Register a programmer `{ label }` → `{ roomId }` |
| `DELETE` | `/queue/:roomId` | Deregister a programmer |
| `GET` (WS) | `/rooms/:id/programmer` | WebSocket for the programmer |
| `GET` (WS) | `/rooms/:id/client` | WebSocket for the client |

Two Durable Objects back the worker:

- **`QueueDO`** — singleton, persists the queue of waiting programmers in KV storage
- **`RoomSession`** — one per room, relays messages between programmer and client using hibernatable WebSockets
