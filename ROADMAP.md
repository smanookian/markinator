# Roadmap

## v1 (done)

Edit / view toggle, Omarchy theme, open / save, GitHub release.

## To do

- Upload to AUR (when registrations reopen), so users can `yay -S markinator`.
- Add to stevinator.com/apps.

## Use less resources

Measured with 11 windows open: about 170-230 MB memory per window (mostly the
built-in web engine), about 2 GB total.

- Done: fixed idle CPU. The theme watcher reacted to file reads, so the windows
  kept waking each other up (about 90% of one core with 11 windows).
- One app, many windows: opening another `.md` file adds a window to the running
  app instead of starting a new copy. Windows share one web engine. Saves about
  half the memory per extra window.
- Turn off unused web engine parts (the app never goes online). Small win.
- Free the rendered page when a window is in the background. Small win.

## Ideas for v2

Only add what is really missed after using v1.

- Search in file (`Ctrl+F`).
- Syntax colors in edit mode (headings, bold, code). Needs a small editor library like CodeMirror.
- Export to HTML / PDF.
- Remember scroll position per file.
- Recent files list.
- Word count in the status line.
