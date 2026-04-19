# Break-Time 🍅

A modern Pomodoro timer and productivity app built with **Rust**, **GTK4**, and **Libadwaita**.

## Features

- **Pomodoro Timer**: 25m/50m work sessions with Pause/Resume and native Linux desktop notifications.
- **Persistent RSS Reader**: Unlocked only after completing a work session. Supports multiple feeds and persists your feed list to `~/.config/break-time/feeds.json`.
- **Markdown Notes**: Always-accessible markdown editor with syntax highlighting, custom themes, and a "Save to .md" feature.
- **Reliable Reddit Support**: Automatically fallbacks to stable Redlib instances (`redlib.catsarch.com`) to bypass Reddit's bot-detection filters.

## Prerequisites

You'll need the GTK4 and Libadwaita development headers installed on your Linux system:

```bash
# Arch Linux
sudo pacman -S gtk4 libadwaita sourceview5

# Fedora
sudo dnf install gtk4-devel libadwaita-devel gtksourceview5-devel

# Ubuntu/Debian
sudo apt install libgtk-4-dev libadwaita-1-dev libgtksourceview-5-dev
```

## How to Run

```bash
cargo run
```

## Why?

Break-Time is designed to help you focus. By locking your distracting RSS feeds (Reddit, news, etc.) behind a work timer, you can ensure you've earned your break before you dive into the news.
