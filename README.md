# MyNote

A simple, fast note app for Linux. Runs natively on GNOME, so it looks and feels like part of your desktop.

Built with **Rust**, **GTK4** and **Libadwaita**.

## Why MyNote

It starts in a blink, uses almost no memory, and saves as you type — so you never lose a thought. There's no cloud, no account, no subscription. Your notes stay on your machine.

## What it does

- **Saves automatically** – no save button needed, but there's one anyway.
- **Write in Markdown** – with a live preview, a split view, and a small toolbar for the common stuff.
- **Stay organised** – tag notes, pin the important ones, and search across everything instantly.
- **Trash bin** – deleted notes go to Trash, so you can bring them back if you change your mind.
- **Export and import** – move notes in and out as Markdown (`.md`) or plain text (`.txt`), one at a time or all at once.
- **Remembers your place** – reopens on the note you were working on, in the view you left it in.

## Install

### Ubuntu / Debian

```bash
sudo apt install libgtk-4-dev libadwaita-1-dev
```

### Fedora

```bash
sudo dnf install gtk4-devel libadwaita-devel
```

### Arch

```bash
sudo pacman -S gtk4 libadwaita
```

### Build

```bash
cargo build --release
```

Run it:

```bash
./target/release/mynote
```

Install it system-wide (for the app menu and desktop search):

```bash
./scripts/install.sh
```

## Keyboard shortcuts

| What it does | Keys |
|---|---|
| New note | `Ctrl + N` |
| Search notes | `Ctrl + F` |
| Save now | `Ctrl + S` |
| Pin / un-pin note | `Ctrl + D` |
| Toggle preview | `Ctrl + P` |
| Export note | `Ctrl + E` |
| Import note | `Ctrl + I` |
| Move note to trash | `Ctrl + Delete` |
| Bold text | `Ctrl + B` |
| Insert a link | `Ctrl + K` |
| Insert a code block | `Ctrl + Shift + C` |
| See all shortcuts | `Ctrl + ?` |
| Quit | `Ctrl + Q` |

## Where notes are stored

Everything lives in one JSON file in your user data folder:

```
~/.local/share/mynote/notes.json
```

Delete that file (or the whole `mynote` folder) to wipe your data. Keeping a copy is a quick way to back up your notes.

## License

MIT. See [LICENSE](LICENSE).
