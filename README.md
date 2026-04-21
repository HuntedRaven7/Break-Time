# Break-Time 🍅

A modern Pomodoro timer and productivity app built with **Rust**, **GTK4**, and **Libadwaita**.

## PLEASE READ THIS

If you are uncomfortable with the use of AI in software please be aware that I used AI to help make this.

## Features

- **Pomodoro Timer**: 25m/50m work sessions with Pause/Resume and native Linux desktop notifications.
- **Persistent RSS Reader**: Unlocked only after completing a work session. Supports multiple feeds and persists your feed list to `~/.config/break-time/feeds.json`.
- **Markdown Notes**: Always-accessible markdown editor with syntax highlighting, custom themes, and a "Save to .md" feature.
- **Reliable Reddit Support**: Automatically fallbacks to stable Redlib instances (`redlib.catsarch.com`) to bypass Reddit's bot-detection filters.

## Prerequisites

### System Libraries
You'll need the GTK4 and Libadwaita development headers installed on your Linux system:

```bash
# Arch Linux
sudo pacman -S gtk4 libadwaita gtksourceview5

# Fedora
sudo dnf install gtk4-devel libadwaita-devel gtksourceview5-devel

# Ubuntu/Debian
sudo apt install libgtk-4-dev libadwaita-1-dev libgtksourceview-5-dev
```

### Flatpak Tools
To build the Flatpak, you also need:
- `flatpak`
- `flatpak-builder`
- `appstream` (for metadata validation)
- `librsvg` and `gdk-pixbuf` tools (required by `appstreamcli` to process icons)

```bash
# Ubuntu/Debian
sudo apt install flatpak flatpak-builder appstream librsvg2-bin libgdk-pixbuf2.0-bin

# Fedora
sudo dnf install flatpak flatpak-builder appstream librsvg2 libgdk-pixbuf2

# Arch Linux
sudo pacman -S flatpak flatpak-builder appstream librsvg gdk-pixbuf2
```

## How to Run

```bash
cargo run
```

## Flatpak

You can build and install Break-Time as a Flatpak using the provided automation script:

```bash
# This script will automatically set up the environment and build the app
./build-flatpak.sh
```

Alternatively, you can build manually with `flatpak-builder`:

```bash
# Build and install locally
flatpak-builder --user --install --force-clean build-dir io.github.HuntedRaven7.BreakTime.yml
```

Run the application:

```bash
flatpak run io.github.HuntedRaven7.BreakTime
```

## Why?

Break-Time is designed to help you focus. By locking your distracting RSS feeds (Reddit, news, etc.) behind a work timer, you can ensure you've earned your break before you dive into the news.
