# midi-daw

A python library that is desined to support live coding musical preformances. There are a few main ways of utilizing this library.

1. The first is best suited for a [tmux](https://github.com/tmux/tmux/wiki) & terminal text editor based approach (see [recomended editors](#recomended-editors)). This involves setting the MIDI output device for the whole file; meaning that all midi out put for the whole file gets sent to that output. 
2. Using decorators to define the midi output device and channel per function. This is best suited for a situation where all code is in a single file, or for when using [Jupyter notebook](https://jupyter.org/).
3. Control it via an andoroid app. (see: `./frontends/midi-daw.android/` for more on that..)

## Recomended Editors

these are listen in no particular order.

- [neovim](https://neovim.io/)
- [emacs](https://www.gnu.org/software/emacs/download.html) in [no-window mode](https://www.gnu.org/software/emacs/manual/html_node/emacs/Initial-Options.html#index-_002d_002dno_002dwindow_002dsystem)
- [helix](https://helix-editor.com/)
- [micro](https://micro-editor.github.io/)
- [Jupyter Notebook](https://jupyter.org/)

## How it Works

The main midi-daw program runs as a webserver accessable from the local network and a [Unix Domain Socket](https://en.wikipedia.org/wiki/Unix_domain_socket). This webserver serves an API that allows for sending MIDI messages to specific usb, or vitual, MIDI devices, and play-back/edditing of sequences. The user can then use an android app or python code to control the sequences, or direct playback. The android app can be used to create sequences in a tracker like interface for melodic tracks, or a drum sequencer interface for drum tracks.

## Features

1. python library for music creation with code.
2. auto detect new midi devices.
3. virtual midi device creation.
4. server side sequencer.
5. android app for control.
6. control from multiple devices at once.
7. saving and loading sessions and tracks.

## TODO

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
- [ ] make automations run server-side
- [x] add an optional "setup" function that must run before a decorated function with looping enabled can start looping
- [x] create virtual midi-outs
- [x] make the sequencer run server-side to minimizes latency.
- [ ] add stop-all button to app
- [x] add queue-stop functionality to server
- [x] add queue-stop button to app
- [x] add step count hilighting to the app
  - [x] connect with websocket
  - [x] get step
- [x] add saving/loading/listing/rm-ing of sequences
  - [x] saving
  - [x] listing
  - [x] loading
  - [x] rm-ing of saved data
  - [x] mk api end-points
- [x] add saving/loading/listing/rm-ing of entire-projects
  - [x] saving
  - [x] listing
  - [x] loading
  - [x] rm-ing of saved data
  - [x] mk api end-points
- [x] add message bus notifications enum for common events, sent as binary messages and encoded with the bincode (v2.0.1) crate
  - [x] change android app to use the new message format
- [x] add BPQ setting & getting api end points
- [ ] set up the server as a wifi access point to remove the need for a travel router
