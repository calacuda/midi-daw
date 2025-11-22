_:
  @just -l

_new-tmux-dev-session SESSION:
  tmux new -ds "{{SESSION}}" -n "README"
  tmux send-keys -t "{{SESSION}}":README 'nv ./README.md "+set wrap"' ENTER
  @just _new-window "{{SESSION}}" "Server" "cd midi-daw-server && nv src/{main.rs,**/*.rs}"
  @just _new-window "{{SESSION}}" "Data Types" "cd midi-daw-types && nv src/lib.rs src/automation/{mod.rs,**/{mod.rs,*.rs}}"
  @just _new-window "{{SESSION}}" "Edit Py" "cd python-lib && nv midi_daw/{main.py,__init__.py}"
  @just _new-window "{{SESSION}}" "Run" "cd python-lib"
  @just _new-window "{{SESSION}}" "Misc" ""
  @just _new-window "{{SESSION}}" "Git" "git status"
  # @just _new-window "{{SESSION}}" "py-front" "cd ./frontends/midi-daw.knulli/ && nv midi-daw.pygame"
  # @just _new-window "{{SESSION}}" "test-front" "cd ./frontends/midi-daw.knulli/ "
  # @just _new-window "{{SESSION}}" "rust-front" "cd ./frontends/midi-daw.knulli/midi_daw_back/ && nv ./src/{lib.rs,*.rs,**/*.rs}"
  # @just _new-window "{{SESSION}}" "rust-front-build" "cd ./frontends/midi-daw.knulli/midi_daw_back/"
  # # @just _new-window "{{SESSION}}" "Git" "git status"

_new-window SESSION NAME CMD:
  tmux new-w -t "{{SESSION}}" -n "{{NAME}}"
  tmux send-keys -t "{{SESSION}}":"{{NAME}}" ". ./.venv/bin/activate" ENTER
  [[ "{{CMD}}" != "" ]] && tmux send-keys -t "{{SESSION}}":"{{NAME}}" "{{CMD}}" ENTER || true

_new-tmux-dev-session-2 SESSION:
  tmux new -ds "{{SESSION}}" -n "Edit-1"
  tmux send-keys -t "{{SESSION}}":"Edit-1" '. ./.venv/bin/activate' ENTER
  tmux send-keys -t "{{SESSION}}":"Edit-1" 'cd ./test-files/' ENTER
  @just _new-window "{{SESSION}}" "Run-1" "cd ./test-files/"
  @just _new-window "{{SESSION}}" "Edit-2" "cd ./test-files/"
  @just _new-window "{{SESSION}}" "Run-2" "cd ./test-files/"
  @just _new-window "{{SESSION}}" "Misc" "cd ./test-files/"

tmux:
  tmux has-session -t '=midi-daw' || just _new-tmux-dev-session midi-daw
  tmux has-session -t '=midi-daw-test' || just _new-tmux-dev-session-2 midi-daw-test
  tmux a -t '=midi-daw'

tmux-2:
  tmux has-session -t '=midi-daw-test' || just _new-tmux-dev-session-2 midi-daw-test
  tmux a -t midi-daw-test

play-note:
  http -j post http://127.0.0.1:8080/midi --raw '{"midi_dev":"OUTPUT-DEVICE","channel":"Ch1","msg":{"PlayNote":{"note":42,"velocity":100,"duration":{"Sn":1}}}}'

get-devs:
  http http://127.0.0.1:8080/midi

backup:
  git status
  git add .
  @just commit "backup commit"
  git push

jupyter:
  bash -c "source ./.venv/bin/activate && jupyter-lab"

adb:
  adb reverse tcp:8080 tcp:8080

test-env:
  bash -c "tmux has-session -t \"=midi-daw-test-run\" && bash -c \"tmux a -t \"midi-daw-test-run\"; exit 1\" || true "
  @just adb
  tmux new -ds "midi-daw-test-run" -n "jupyter" "just jupyter"
  tmux new-window -t "midi-daw-test-run" -n "midi-daw-server" "cd ./midi-daw-server && cargo run -r"

_do_commit:
  # ./frontends/midi-daw.knulli/.venv/bin/pip list --format=freeze -l | cut -d "=" -f 1 | rg -v pip > ./frontends/midi-daw.knulli/requirements.txt
  ./frontends/midi-daw.knulli/.venv/bin/pip freeze -l > ./frontends/midi-daw.knulli/requirements.txt
  git add frontends/midi-daw.knulli/requirements.txt

commit message: _do_commit
    git commit -m "{{message}}"
    git push

commit-all message: _do_commit
    git commit -am "{{message}}"
    git push
  

