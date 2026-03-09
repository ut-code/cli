# ut.code(); CLI

A terminal UI for browsing the [ut.code();](https://utcode.net) community.

## Install

```sh
cargo install --git https://github.com/ut-code/cli
```

## Usage

```sh
utcode
```

Launches a full-screen interactive TUI with four categories:

| Category | Contents |
|----------|----------|
| About    | Organization overview, activities, tech stack, and contact info |
| Projects | Active projects built by ut.code(); members |
| Articles | Recent articles and event reports from utcode.net |
| Members  | Current members and their areas of work |

### Navigation

| Key | Action |
|-----|--------|
| `Tab` / `→` / `l` | Move focus to the next panel |
| `Shift+Tab` / `←` / `h` | Move focus to the previous panel |
| `↓` / `j` | Select next item |
| `↑` / `k` | Select previous item |
| `q` / `Esc` | Quit |
