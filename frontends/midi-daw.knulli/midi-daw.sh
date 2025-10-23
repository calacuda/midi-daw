#!/bin/bash

dir="/userdata/roms/ports/midi-daw/"
cd $dir
# . ./.venv/bin/activate
./.venv/bin/python ./midi-daw.pygame 2>&1 > ./logs.txt
