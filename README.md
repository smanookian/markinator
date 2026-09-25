# markinator

Markdown editor and reader for Omarchy.

One window. Write markdown. Press `Ctrl+E`. Read it rendered.
Press `Ctrl+E` again. Back to writing.

It uses your Omarchy theme colors and font. Change the theme and the app follows, no restart.

## Install

One line, no sudo. Puts the binary in `~/.local/bin` and adds the launcher entry and icon:

```
curl -fsSL https://raw.githubusercontent.com/smanookian/markinator/main/install.sh | sh
```

Read [install.sh](install.sh) first if you like; it is short.

### Build from source

Needs Rust and `webkit2gtk-4.1` (already on Omarchy).

```
git clone https://github.com/smanookian/markinator
cd markinator/src-tauri
cargo build --release
install -Dm755 target/release/markinator ~/.local/bin/markinator
install -Dm644 ../markinator.desktop ~/.local/share/applications/markinator.desktop
install -Dm644 ../icons/markinator.svg ~/.local/share/icons/hicolor/scalable/apps/markinator.svg
```

Or build the Arch package: `makepkg -si` in the repo root (needs a released tag; it downloads the tarball).

Make it the default for markdown files:

```
xdg-mime default markinator.desktop text/markdown
```

Note: on Hyprland, `xdg-open` asks `file` for the type, and `file` calls `.md` files
`text/plain`. Install `perl-file-mimeinfo` (`sudo pacman -S perl-file-mimeinfo`) and
`xdg-open notes.md` will pick markinator. `gio open notes.md` works without it.

## Use

```
markinator               # empty document
markinator notes.md      # open a file (starts in view mode)
markinator -e notes.md   # open a file in edit mode
markinator --version     # show the version
markinator --help        # show options and keys
```

## Keys

| Key | Action |
|-----|--------|
| `Ctrl+E` | Toggle edit / view |
| `Ctrl+O` | Open |
| `Ctrl+S` | Save |
| `Ctrl+Shift+S` | Save as |
| `Ctrl+N` | New document |
| `Ctrl+Q` | Quit |
| `Ctrl++` / `Ctrl+-` / `Ctrl+0` | Zoom in / out / reset |

In edit mode, `Tab` inserts two spaces.

## Notes

- Links to `http(s)://` open in your browser. Links to `.md` files open in the same window.
- If the file changes on disk and you have no unsaved edits, it reloads.
- Window size is remembered. Nothing else is.
- Window class is `markinator`, for Hyprland window rules.

## License

MIT
