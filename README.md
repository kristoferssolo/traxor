# Traxor

A terminal UI for managing Transmission torrents.

## Features

- Vim-style navigation (`hjkl`)
- Mouse support: click to select a torrent or tab, scroll to move the highlight
- Live fuzzy search/filter
- Custom tab layouts with configurable columns
- Multi-select for batch operations
- Move, rename, delete torrents
- Real-time transfer statistics
- Fully configurable keybinds and colors

## Screenshots

| All torrents | Seeding torrents |
|:--:|:--:|
| ![All torrents](assets/all.png) | ![Seeding torrents](assets/seeding.png) |

## Installation

```bash
cargo binstall traxor
```

Or build from source:

```bash
git clone https://github.com/kristoferssolo/traxor
cd traxor
cargo build --release
```

## Usage

Make sure Transmission daemon is running, then:

```bash
traxor
```

### Keybinds

| Key | Action |
|-----|--------|
| `j/k` | Navigate up/down |
| `g/G` | First/last torrent |
| `PgUp/PgDn` | Page up/down |
| `h/l` | Previous/next tab |
| `1-9, 0` | Switch to tab |
| `Enter` | Start/stop torrent |
| `a` | Start/stop all |
| `Space` | Multi-select |
| `Ctrl+a` | Select all visible |
| `Ctrl+u` | Clear selection |
| `m` | Move torrent |
| `r` | Rename torrent |
| `d` | Delete torrent |
| `D` | Delete with data |
| `/` | Search/filter |
| `Esc` | Close popup / clear filter |
| `s` | Sort by next column |
| `S` | Reverse sort |
| `?` | Open/close the full help page |
| `q` | Quit |

In move, rename, and filter prompts:

| Key | Action |
|-----|--------|
| `Left/Right` | Move cursor |
| `Home/End`, `Ctrl+A/E` | Line start/end |
| `Delete` | Delete forward |
| `Ctrl+W` | Delete previous word |
| `Ctrl+U` | Delete to line start |

## Configuration

Configuration file: `~/.config/traxor/config.toml`

Only specify values you want to override. See [config/default.toml](config/default.toml) for all options.

### RPC

```toml
[rpc]
url = "http://localhost:9091/transmission/rpc"
username = "transmission"
password = "secret"
```

`url` defaults to `http://localhost:9091/transmission/rpc` and must use `http` or `https`. `username` and `password` enable HTTP basic auth and must be set together.

While credentials are set, `[log] transmission_rpc` is capped at `info`, because transmission-rpc logs the password at `debug`.

### Custom Tabs

```toml
[[tabs]]
name = "My Tab"
columns = ["status", "progress", "name", "size"]
statuses = ["Downloading", "QueuedToDownload"]
```

Available columns: `name`, `status`, `size`, `downloaded`, `uploaded`, `ratio`, `progress`, `eta`, `peers`, `seeds`, `leeches`, `downspeed`, `upspeed`, `path`, `added`, `done`, `left`, `queue`, `error`, `labels`, `tracker`, `hash`, `private`, `stalled`, `finished`, `files`, `activity`
Available statuses: `Stopped`, `QueuedToVerify`, `Verifying`, `QueuedToDownload`, `Downloading`, `QueuedToSeed`, `Seeding`

### Colors

```toml
[colors]
highlight_background = "#3a3a5a"
highlight_foreground = "white"
status_downloading = "cyan"
status_seeding = "white"
status_stopped = "dark_gray"
progress_downloading = "#163a40"
progress_seeding = "#262626"
progress_stopped = "#1c1c1c"
progress_verifying = "#3a3220"
progress_queued = "#1c2a3a"
```

Each row's Name cell is filled from the left in proportion to the torrent's progress, using the `progress_*` color for its state. On the highlighted or selected row, hex colors are brightened by 30% so the row background stays distinguishable; named colors are used as is.

### Time

```toml
[time]
date_format = "%Y-%m-%d %H:%M"
eta_format = "compact"
```

`date_format` uses chrono strftime syntax and is rendered in local time. `eta_format` accepts `compact` or `seconds`.

### Keybinds

```toml
[keybinds]
quit = "q"
next_torrent = "j"
prev_torrent = "k"
filter = "/"
```

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT) at your option.
