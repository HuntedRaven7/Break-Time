# Break-Time 🍅

A modern Pomodoro timer and productivity app built with **Rust**, **GTK4**, and **Libadwaita**.

## PLEASE READ THIS

If you are uncomfortable with the use of AI in software please be aware that I used AI to help make this.

## Features

- **Pomodoro Timer**: Standard 25m/50m work sessions, plus a **Custom Timer** with hours, minutes, and seconds support. Includes native Linux desktop notifications.
- **Persistent RSS Reader**: Multiple feeds with local persistence; use the RSS tab any time. One-click refresh loads recent articles from your saved URLs.
- **Integrated Todo & Notes**: A unified, resizable workspace for managing tasks and writing notes side-by-side. 
- **Linked Task-Notes**: Instantly bind Markdown files to specific Todo items. Click the "📝" icon on any task to open its associated note instantly.
- **Tabbed Markdown Editor**: Advanced editor supporting multiple open files simultaneously with syntax highlighting, a "Pin Active Note" feature, and persistent theme selection.
- **Workspace Resizing**: Smoothly adjust the focus of your workspace by dragging the boundary between tasks and notes, or collapse the sidebar entirely for distraction-free writing.
- **System-Adaptive Themes**: Editor themes automatically sync with system-wide light/dark preferences, ensuring a consistent visual experience.
- **Interactive Calendar**: Split-view calendar with persistent event management and visual markers for any date.
- **Reliable Reddit Support**: Automatic fallback to stable Redlib instances (`redlib.catsarch.com`) to ensure uninterrupted feed access.

## Data Persistence

Your data is saved locally for easy backup and portability. On Linux, your files are located at:

- **Todo Tasks**: `~/.local/share/BreakTime/todos.json`
- **Calendar Events**: `~/.local/share/BreakTime/calendar_events.json`
- **RSS Feeds**: `~/.config/break-time/feeds.json`
- **Note Settings**: `~/.config/break-time/note_settings.json` (Stores your preferred editor theme)


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

THIS IS REALLY THE RECOMMENDED WAY OF INSTALLING

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

Break-Time combines a straightforward Pomodoro timer with notes, tasks, calendar, and RSS in one window—so you can time work sessions and switch to reading or planning without juggling separate apps.