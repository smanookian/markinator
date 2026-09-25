# Roadmap

## v1 (done)

Edit / view toggle, Omarchy theme, open / save, GitHub release.

## To do

- Upload to AUR (when registrations reopen), so users can `yay -S markinator`.
- Done: added to stevinator.com/apps.
- Done (v0.1.2): command line flags `--version` (`-v`), `--help`, `--edit`.
- Terminal view (`markinator -t file.md`), like glow. See notes below.

## Use less resources

Measured with 11 windows open: about 170-230 MB memory per window (mostly the
built-in web engine), about 2 GB total.

- Done: fixed idle CPU. The theme watcher reacted to file reads, so the windows
  kept waking each other up (about 90% of one core with 11 windows).
- Done: one app, many windows. Opening another `.md` file adds a window to the
  running app instead of starting a new copy. Measured: first window about 230 MB,
  each extra window about 100 MB (before: about 200 MB each).
- Checked, not worth it: turn off unused web engine parts. The network process
  is about 26 MB, only one for all windows, and WebKit needs it for local files too.
- Checked, not worth it: free the rendered page when a window is in the background.
  Most of the ~100 MB per window is the empty web engine, not the note.

## Ideas for v2

Only add what is really missed after using v1.

- Done: search in file (`Ctrl+F`).
- Syntax colors in edit mode (headings, bold, code). Needs a small editor library like CodeMirror.
- Export to HTML / PDF.
- Remember scroll position per file.
- Recent files list.
- Done: word count in the status line.

## Notes: terminal view (`-t`)

Possible. Two ways:

- Easy: `-t` runs `glow` if it is installed. Few lines, but needs glow.
- Own: turn markdown into colored terminal text in Rust (headings, bold, lists,
  code, quotes, links). A few hundred lines, no new app needed. Tables and images
  are harder in a terminal. Can use the Omarchy colors too.

Read only. Editing in the terminal would be a second editor, too much for now.
