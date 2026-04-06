# from midi_daw import *
import time

from midi_daw import MidiChannel, NoteLen, v2
from midi_daw.v2 import lfo, main
from midi_daw.v2.note_lens import *

print("\n\n", "=" * 30, "\n\n", sep="")
# song = v2.MidiDaw("Vital", MidiChannel.Ch1, virt=True)
# automa = v2.MidiDaw("Vital", MidiChannel.Ch1, virt=True)
drums = v2.MidiDaw("808", MidiChannel.Ch10, virt=True)

devs = v2.list_devs()
print(devs)


@drums.register
def f(api):
    # # print(f"func-name = {f.__name__}")
    # api.play("g4", NoteLen.Sn(7))
    # # print(f"py: f playing  playing g4 on {api.device}:{api.channel} again", flush=True)
    # # api.play("g4", NoteLen.Wn(1))
    # # time.sleep(1.0)
    # # print(f"py: playing g4 on {api.device}:{api.channel}")
    # api.rest(NoteLen.Sn(1))
    # api.play("c4", sn(1))
    api.play("c2", qn(1))
    # api.rest()


@main
def main():
    # loops = 128
    loops = 0
    f_jh = f(loops=loops, block=True)
    print("non-blockers called")
    drums.stop()
    print("blocking thread done")
    print("exiting")
    exit(0)
