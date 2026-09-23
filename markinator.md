# markinator - full spec

Markdown editor and reader for Omarchy (Arch Linux, Hyprland, Wayland).
Published at https://stevinator.com/apps/ . Free. MIT.

Give this whole file to the AI that builds the app.

---

## 1. Goal

One window. Write markdown. Press `Ctrl+E`. Read it rendered.
Press `Ctrl+E` again. Back to writing. Nothing else in the way.

It should feel like a native Omarchy app: same colors, same font, keyboard first.

## 2. Rules for the builder

- Simple english everywhere: README, UI text, comments, commit messages. Keep it short.
- Ask if unsure. No guessing.
- Keep the code small. No extra features. No abstractions that are not needed.
- No JS framework. No CSS framework. No build step for the frontend beyond what Tauri needs.
- License MIT. Add `LICENSE` and `README.md` (what it is, install, keys).
- Name: `markinator`. Binary: `markinator`. App id: `com.stevinator.markinator`.

## 3. Stack

Tauri 2. Rust backend. Frontend: one `index.html`, one `style.css`, one `main.js`.

Why:
- Markdown renders best in a webview (headings, tables, code, images).
- Theme colors map straight to CSS variables.
- Wayland native through webkit2gtk. Already on Arch.
- One binary. Easy to package for AUR later.

Rejected: egui / iced (weak markdown rendering). GTK4 TextView (rendering markdown by hand is a lot of code).

Crates (Rust):
- `tauri` 2
- `tauri-plugin-dialog` (open / save dialogs)
- `tauri-plugin-cli` (file argument)
- `pulldown-cmark` (markdown to HTML, GFM: tables, strikethrough, task lists)
- `toml` + `serde` (read `colors.toml`)
- `notify` (watch theme dir)

Frontend editor: a plain `<textarea>`. No CodeMirror in v1. Syntax highlighting in the editor is **not** in v1.

Code block highlighting in view mode: `highlight.js`, bundled locally (no CDN), one theme file recolored by CSS variables.

## 4. Features (v1)

### 4.1 Modes
- **Edit mode**: full-window `<textarea>`. Monospace font. Line wrap on.
- **View mode**: full-window rendered markdown. Scrollable. Read only.
- `Ctrl+E` toggles. Scroll position: keep roughly the same place (same line ratio). Best effort.
- Start in **view mode** if a file was given on the command line. Start in **edit mode** if not.

### 4.2 Files
- `markinator file.md` opens the file. Path can be relative or absolute.
- `markinator` with no file: empty document, untitled.
- `Ctrl+O`: open file dialog. Filter `*.md, *.markdown, *.txt`.
- `Ctrl+S`: save. If untitled, acts like save as.
- `Ctrl+Shift+S`: save as.
- Encoding: UTF-8 only.
- Relative image and link paths resolve from the file's folder.
- Links: `http(s)://` open in the system browser. `.md` links open in the same window (replaces the document, asks if unsaved).
- Unsaved changes on close (`Ctrl+Q` or window close): dialog "Save changes?" with Save / Don't save / Cancel.
- If the file changes on disk while open and there are no unsaved edits: reload it silently.

### 4.3 Window
- Title: `name.md - markinator`. Unsaved: `*name.md - markinator`. Untitled: `Untitled - markinator`.
- Remember window size between runs (Tauri window state plugin or a tiny config file). Nothing else is remembered.
- No menu bar. No toolbar. A small status line at the bottom: mode (`EDIT` / `VIEW`), file name, `Ctrl+E to toggle`. Hide it after 2 seconds of no key or mouse, show again on any key or mouse move.

### 4.4 Keys
| Key | Action |
|-----|--------|
| `Ctrl+E` | Toggle edit / view |
| `Ctrl+O` | Open |
| `Ctrl+S` | Save |
| `Ctrl+Shift+S` | Save as |
| `Ctrl+N` | New empty document (asks if unsaved) |
| `Ctrl+Q` | Quit (asks if unsaved) |
| `Ctrl++` / `Ctrl+-` / `Ctrl+0` | Zoom in / out / reset (both modes) |
| `Esc` | In view mode: nothing. In edit mode: nothing. (Esc must not quit.) |

Keys work in both modes. The textarea keeps normal text keys (Tab inserts two spaces).

### 4.5 Omarchy theme
- Theme dir: `~/.local/state/omarchy/current/theme/`. Colors: `colors.toml` in that dir.
- Read these keys: `mode`, `accent`, `selection`, `muted`, `background`, `dark_background`, `darker_background`,
  `lighter_background`, `foreground`, `dark_foreground`, `light_foreground`, `bright_foreground`,
  `red`, `yellow`, `orange`, `green`, `cyan`, `blue`, `magenta`.
- Map to CSS variables `--accent`, `--background`, ... (same names, dashes for underscores).
- Font: run `fc-match monospace -f '%{family}'`, take the first name before the comma. Use it for the editor
  and for code blocks. Body text in view mode: same font (Omarchy is monospace everywhere). Size 15px default.
- Watch the theme dir with `notify`. When `colors.toml` changes, re-read it and push new colors to the frontend. No restart.
- If `colors.toml` is missing (not Omarchy): fall back to a built-in dark set (use the Catppuccin values below).
- Fallback colors:
  ```
  background=#1e1e2e  lighter_background=#313244  darker_background=#101019
  foreground=#cdd6f4  dark_foreground=#6c7086  accent=#89b4fa  selection=#45475a  muted=#585b70
  red=#f38ba8 yellow=#f9e2af orange=#f6b6ab green=#a6e3a1 cyan=#94e2d5 blue=#89b4fa magenta=#f5c2e7
  ```

Color use in view mode:
- page: `background` / `foreground`
- headings: `accent`
- links: `blue`, underline on hover
- inline code and code blocks: `lighter_background` bg, `foreground` text
- blockquote: left border `muted`, text `light_foreground`
- table borders: `muted`, header row bg `lighter_background`
- hr: `muted`
- selection: `selection`
- task list checkbox checked: `green`

Edit mode: `background` / `foreground`, caret `accent`, selection `selection`.

### 4.6 Desktop integration
- Ship `markinator.desktop`:
  ```
  [Desktop Entry]
  Name=markinator
  Comment=Markdown editor and reader
  Exec=markinator %f
  Icon=markinator
  Terminal=false
  Type=Application
  Categories=Utility;TextEditor;
  MimeType=text/markdown;text/x-markdown;
  ```
- Ship an icon (SVG, simple, uses no more than 2 colors). Put it under `hicolor/scalable/apps/markinator.svg`.
- README shows how to set it as default: `xdg-mime default markinator.desktop text/markdown`.
- Hyprland: the window class must be `markinator` so users can write window rules.

### 4.7 Not in v1
Tabs. File tree. Split view. Editor syntax highlighting. Search in file. Export to PDF/HTML. Spell check. Plugins. Sync. Settings UI.

## 5. Behaviour details

- Rendering happens in Rust (`pulldown-cmark` -> HTML string) and is sent to the frontend on every toggle to view mode. Not on every keystroke.
- Raw HTML inside markdown: render it (like GitHub). Scripts must not run: set a strict CSP in `tauri.conf.json` (`script-src 'self'`).
- Large files (5 MB+): must still open. Rendering may be slow, that is fine.
- Empty file in view mode: show muted text `Empty. Press Ctrl+E to write.`
- File not found on startup: open empty editor with that path as the target, so `Ctrl+S` creates it. Show status line message `New file`.
- File not readable (permissions, binary): show an error dialog, start empty.

## 6. Project layout

```
markinator/
  README.md
  LICENSE
  markinator.desktop
  icons/markinator.svg
  src/            frontend: index.html, style.css, main.js, vendor/highlight.min.js, vendor/highlight.css
  src-tauri/      Rust: src/main.rs, src/theme.rs, src/files.rs, Cargo.toml, tauri.conf.json, capabilities/
  PKGBUILD        Arch package (build from source). Optional in v1, nice to have.
```

## 7. Done means

All of these work on a real Omarchy machine:
1. `markinator README.md` opens in view mode, rendered correctly (headings, list, table, code block).
2. `Ctrl+E` switches to edit mode, `Ctrl+E` again back to view. Edits show up in view.
3. `Ctrl+S` writes the file. Title `*` goes away.
4. `Ctrl+O` opens another file.
5. `omarchy theme set tokyo-night` while the app is open: colors change without restart. Set it back.
6. Closing with unsaved edits asks first.
7. `xdg-open test.md` launches markinator after `xdg-mime default markinator.desktop text/markdown`.
8. `cargo build --release` gives one binary. No warnings.

## 8. Research (2026-09-23)

Existing apps with an edit / view toggle:
- Ghostwriter (KDE, Qt): `Ctrl+M` HTML preview. Split pane, heavy deps.
- Ferrite (Rust, egui): lightweight, no clear toggle key.
- Remarkable (Python, GTK3): live preview pane, old.
- Terminal: MarkLn (`Ctrl+T`), tui-md-editor (`Ctrl+E`), md-tui (external editor), glow (view only).
None do a clean one-key full-window view mode with Omarchy theming.

Name check: `markinator` is free in Arch repos and AUR. GitHub has 9 small unrelated repos, no released app.
`mdview` was rejected: already an AUR package.
