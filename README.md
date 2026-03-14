# Coding Human CLI

A terminal-based tool to connect with available programmers for Q&A sessions via WebSocket.

## Features

- **TUI Queue Browser** — View available programmers in a live queue and connect to one with a question
- **Programmer Registration** — Register as a programmer to answer questions from users
- **Streaming Responses** — Real-time response streaming with multi-round Q&A support
- **WebSocket-based** — Built on Cloudflare Workers + Durable Objects backend for live connections

## Install

```sh
cargo install --git https://github.com/ut-code/cli
```

## Usage

### Default (TUI Queue Browser)

```sh
coding-human
```

Launches an interactive TUI to:
1. Fetch the live queue of available programmers
2. Select a programmer
3. Ask a question
4. Stream the response in real-time
5. Ask follow-up questions or switch programmers

**Navigation:**
- `j/k` or `↑/↓` — Move up/down in lists
- `Enter` — Select or send
- `Esc`/`q` — Cancel or quit
- `b` — Go back to programmer list (after response)

### Register as a Programmer

```sh
coding-human join <your-name>
```

Registers you in the queue and enters a loop to:
1. Wait for a user's question
2. Read your response from stdin (Ctrl+D to finish each response)
3. Send it to the user
4. Wait for the next question

### Legacy Commands

```sh
coding-human ask
```
Non-TUI version to select a programmer and ask a question.

```sh
coding-human respond <room-id>
```
Legacy direct response mode using a specific room ID.

## Server

The backend is a Hono + Cloudflare Workers project (`server/` directory).

**Key Routes:**
- `POST /rooms` — Create a new room (returns `{ roomId }`)
- `POST /queue/register` — Register programmer in queue
- `DELETE /queue/{roomId}` — Remove from queue
- `GET /queue` — Fetch all queued programmers
- `WebSocket /rooms/{roomId}/user` — User side of Q&A
- `WebSocket /rooms/{roomId}/programmer` — Programmer side of Q&A

## Environment

Set `SERVER_URL` in `.env` (defaults to `http://localhost:8787`):

```
SERVER_URL=http://localhost:8787
```

The CLI automatically converts `http://` ↔ `ws://` and `https://` ↔ `wss://`.

## Protocol

- User sends a question as text over WebSocket
- Programmer sends response lines
- Programmer sends `[DONE]` as a text message to signal end of response
- Connection stays alive for multiple rounds of Q&A
- `/quit` at the question prompt (non-TUI) or `Esc` in TUI exits the session
