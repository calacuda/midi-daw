# midi-daw

  live coding rewrite

A python library that is desined to support live coding musical preformances. There are two main ways of utilising this library. The first is best suited for a [tmux](https://github.com/tmux/tmux/wiki) & terminal text editor based approach (see [recomended editors](#recomended-editors)). This aproach involves setting the MIDI output device for the whole file; meaning that all midi out put for the whole file gets sent to that output. The other aproach involves using decorators to define the midi output device and channel per function. This approach is best suited for a situation where all code is in a single file, or for when using [Jupyter notebook](https://jupyter.org/).

# Recomended Editors

these are listen in no particular order.

- [neovim](https://neovim.io/)
- [emacs](https://www.gnu.org/software/emacs/download.html) in [no-window mode](https://www.gnu.org/software/emacs/manual/html_node/emacs/Initial-Options.html#index-_002d_002dno_002dwindow_002dsystem)
- [helix](https://helix-editor.com/)
- [micro](https://micro-editor.github.io/)
- [Jupyter Notebook](https://jupyter.org/)

# TODO

- [x] when using decorators create a way to name the thread that a function defines, based on the funtion name.
- [x] name the file based midi threads based on file name
- [x] define shared structs in rust using pyo3
  - [x] make lib
  - [x] define types
- [x] start py venv for project
- [x] build rust api & threads
  - [x] threads
  - [x] midi api
  - [x] message bus
  - [x] tempo get/set
  - [x] add listing of midi devices
- [x] enable sending data to api backend from python
- [ ] automation
  - [ ] LFO
      - [x] wavetable
      - [ ] sin
      - [ ] saws
      - [ ] anti-logs
      - [ ] logs
  - [ ] envelopes
      - [ ] adsr
      - [ ] ar
- [x] add an optional "setup" function that must run before a decorated function with looping enabled can start looping
- [x] create virtual midi-outs
<!-- - [ ] augment websocket-message-bus/http-api with mqtt. -->
- [ ] make the sequencer run server-side to minimizes latency.
