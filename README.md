# coding-human

A CLI tool for real-time Q&A between a client and a programmer over WebSocket.

Programmers register themselves in a queue; clients browse the queue and connect to a programmer of their choice.

## Install

```sh
cargo install --git https://github.com/ut-code/cli
```

## Usage

### Programmer: join the queue

```sh
coding-human serve <LABEL>
```

Registers you in the queue with the given display name, creates a room, and waits for a client to connect. Once connected, questions arrive automatically and you type answers line by line. Press `Ctrl+D` to finish a response and wait for the next question.

Example:

```sh
coding-human serve "Alice (Rust / systems)"
```

### Client: ask a question

```sh
coding-human ask
```

Fetches the list of available programmers, lets you pick one, then opens a session. Type questions at the prompt; answers stream back in real time. Type `/quit` or press `Ctrl+D` to exit.

## Configuration

The server URL defaults to `http://localhost:8787`. Override it with a `SERVER_URL` environment variable or a `.env` file:

```
SERVER_URL=https://your-server.example.com
```
