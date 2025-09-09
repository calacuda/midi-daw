# midi-daw

  live coding rewrite

a bevy gui tracker

# TODO

- [ ] finish implementing navigation of midi device selector menu
- [ ] write display for tempo and the note being played 
- [ ] write settings menu
  - [ ] set tempo
  - [ ] ...
- [x] enable proper display for commands
- [x] MidiCmd struct is no Longer relevant, re-structure data accordingly
- [x] enable universal commands
  - [x] `HOLD` command
  - [x] `CHRD` command
  - [x] `ROLL` command
  - [x] `RPET` command
  <!-- - [ ] `SWNG` command -->
  - [x] `STOP` command
  - [x] `CC--` command
- [ ] write command selection and edit menu 
- [ ] add built in SF2 player
- [ ] make midi/sf2/vst target specific to the playback channel NOT the track. The tracks can be played on an acrtive channel and can be switch out at anytime.
- [ ] add panic button to send all-note-off on add devs.
- [ ] add saving and reloading state
 - [ ] saving
 - [ ] reloading
- [ ] add saving/loading to/from midi file
 - [ ] saving
 - [ ] reloading
<!-- - [ ]  -->
