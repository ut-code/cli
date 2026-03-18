#!/usr/bin/env bash
# Start wrangler in a new pane (right)
PANE1=$(wezterm cli split-pane --right -- bash -c "cd worker && bunx wrangler dev; exec zsh")

# Wait for server to start, then launch coder
sleep 3
PANE2=$(wezterm cli split-pane --pane-id $PANE1 --bottom -- bash -c "cd ~/dev/cli && sleep 1 && cargo run -- coder claude; exec zsh")

# Wait a bit more, then launch client
sleep 1
wezterm cli split-pane --bottom -- bash -c "cd ~/dev/cli && sleep 2 && cargo run -- client tknkaa; exec zsh"
