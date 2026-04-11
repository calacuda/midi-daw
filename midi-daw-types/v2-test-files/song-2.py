from midi_daw import MidiChannel, NoteLen, v2
from midi_daw.v2 import lfo, main
from midi_daw.v2.note_lens import *

lead = v2.MidiDaw("vital-lead", MidiChannel.Ch1, virt=True)
bass = v2.MidiDaw("vital-bass", MidiChannel.Ch1, virt=True)


@bass.register
def f(api):
    api.seq("<c4 <e4*2 b4> <c4*2 g4> g4>", sn())


@lead.register
def f_2(api):
    for _ in range(3):
        api.chord(["d4"], en())
        api.rest(en())

    api.chord(["g4", "d4"], sn())
    api.chord(["c3"], sn())
    api.rest(en())
    api.chord(["a4"], sn())


f_jh = f(loops=1, block=False)
f_2(loops=0, block=True)

