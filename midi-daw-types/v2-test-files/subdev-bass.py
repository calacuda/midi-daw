# from this import s

from midi_daw import MidiChannel, NoteLen, v2
from midi_daw.v2 import lfo, main
from midi_daw.v2.note_lens import *

song = v2.MidiDaw("vital-bass", MidiChannel.Ch1, virt=True)


@song.register
def f(api):
    for _ in range(3):
        api.seq("<f3 c3>", sn())
        api.rest(sn())

    api.rest(en())


@song.register
def f_2(api):
    for _ in range(3):
        api.play("f2", sn())
        api.rest(sn())

    api.rest(en())


f(loops=64, block=False)
f_2(loops=64, block=True)
