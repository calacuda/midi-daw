_:
  @just -l

_new-tmux-dev-session SESSION:
  tmux new -ds "{{SESSION}}" -n "README"
  tmux send-keys -t "{{SESSION}}":README 'nv ./README.md "+set wrap"' ENTER
  @just _new-window "{{SESSION}}" "Server" "cd midi-daw-server && nv src/{main.rs,**/*.rs}"
  @just _new-window "{{SESSION}}" "Data Types" "cd midi-daw-types && nv src/lib.rs"
  @just _new-window "{{SESSION}}" "Edit Py" "cd python-lib && nv midi_daw/main.py"
  @just _new-window "{{SESSION}}" "Run" "cd python-lib"
  @just _new-window "{{SESSION}}" "Misc" ""
  @just _new-window "{{SESSION}}" "Git" "git status"

_new-window SESSION NAME CMD:
  tmux new-w -t "{{SESSION}}" -n "{{NAME}}"
  tmux send-keys -t "{{SESSION}}":"{{NAME}}" ". ./.venv/bin/activate" ENTER
  [[ "{{CMD}}" != "" ]] && tmux send-keys -t "{{SESSION}}":"{{NAME}}" "{{CMD}}" ENTER || true

_new-tmux-dev-session-2 SESSION:
  tmux new -ds "{{SESSION}}" -n "Edit"
  tmux send-keys -t "{{SESSION}}":Edit '. ./.venv/bin/activate' ENTER
  tmux send-keys -t "{{SESSION}}":Edit 'cd ./test-files/' ENTER
  # @just _new-window "{{SESSION}}" "Server" "cd midi-daw-server && nv src/{main.rs,**/*.rs}"
  # @just _new-window "{{SESSION}}" "Data Types" "cd midi-daw-types && nv src/lib.rs"
  # @just _new-window "{{SESSION}}" "Edit Py" "cd python-lib && nv midi_daw/main.py"
  # @just _new-window "{{SESSION}}" "Run" "cd python-lib"
  # @just _new-window "{{SESSION}}" "Misc" ""
  # @just _new-window "{{SESSION}}" "Git" "git status"
  @just _new-window "{{SESSION}}" "Run" "cd ./test-files/"
  @just _new-window "{{SESSION}}" "Misc" "cd ./test-files/"

tmux:
  tmux has-session -t midi-daw || just _new-tmux-dev-session midi-daw
  tmux has-session -t midi-daw-test || just _new-tmux-dev-session-2 midi-daw-test
  tmux a -t midi-daw

tmux-2:
  tmux has-session -t midi-daw-test || just _new-tmux-dev-session-2 midi-daw-test
  tmux a -t midi-daw-test

play-note:
  http -j post http://127.0.0.1:8888/midi --raw '{"midi_dev":"OUTPUT-DEVICE","channel":"Ch1","msg":{"PlayNote":{"note":42,"velocity":100,"duration":{"Sn":1}}}}'

get-devs:
  http http://127.0.0.1:8888/midi

# term-test:
#   tmux has-session -t midi-daw-test || just _new-tmux-session midi-daw-test
#   tmux a -t midi-daw-test

