_:
  @just -l

_new-tmux-session SESSION:
  tmux new -ds "{{SESSION}}" -n "README"
  tmux send-keys -t "{{SESSION}}":README 'nv ./README.md "+set wrap"' ENTER
  # @just new-window "Reff" ""
  @just _new-window "{{SESSION}}" "Edit" ""
  @just _new-window "{{SESSION}}" "Run" ""
  @just _new-window "{{SESSION}}" "Git" "git status"

_new-window SESSION NAME CMD:
  tmux new-w -t "{{SESSION}}" -n "{{NAME}}"
  tmux send-keys -t "{{SESSION}}":"{{NAME}}" "{{CMD}}" ENTER


tmux:
  tmux has-session -t midi-daw || just _new-tmux-session midi-daw
  tmux a -t midi-daw
