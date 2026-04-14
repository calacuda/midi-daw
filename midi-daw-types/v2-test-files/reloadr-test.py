from midi_daw_types import MidiChannel 
from midi_daw_types.v2 import lfo, main, wait_for_bar, list_devs
from midi_daw_types.v2.note_lens import *
from midi_daw_test import play_on
from reloadr import autoreload


devs = list_devs()
# print(devs)


@play_on("Vital - bass", is_virt=False)
def bass(m):
    m.seq("c3 <e3*2 b3> <c3*2 g3> ~ g3", sn()) 
    # for chord in ["c4", "e4", "c4", "~", "g4", "c4", "e4", "c4", "~", "g4", "c4", "b4", "g4", "~", "g4"]:
    #     m.chord(chord[0])
    wait_for_bar()


@play_on("909", MidiChannel.Ch10, is_virt=False)
def kick(m):
    # m.seq("c2 [c2 e2] c2 [c2 e2]", qn())
    notes = ["c2", "c2 e2", "c2", "c2 e2"]

    for chord in notes:
        m.chord(chord, qn())


@play_on("909", MidiChannel.Ch10, is_virt=False)
def hh(m):
    m.seq("f#2 f#2 a#2 ~ ~ f#2 f#2 a#2 f#2 f#2 a#2 ~ ~ f#2 f#2 a#2", sn())
    # steps = "f#2 f#2 a#2 ~ ~ f#2 f#2 a#2 f#2 f#2 a#2 ~ ~ f#2 f#2 a#2".split(" ")
    # steps = ["f#2"] * 16

    # for step in steps:
    #     # print(step)
    #     m.note(step, sn())
    # print("done")


@play_on("Vital - lead", is_virt=False)
def lead(m):
    m.seq("c4 <e4*2 b4> <c4*2 g4> ~ g4", sn()) 
    # for chord in ["c4", "e4", "c4", "~", "g4", "c4", "e4", "c4", "~", "g4", "c4", "b4", "g4", "~", "g4"]:
    #     m.chord(chord[0])
    wait_for_bar()


loops = 32


# @main
# def main():
#     bass(loops=loops, block=False)
#     lead(loops=loops, block=False)
#     kick(loops=loops, block=False)
#     hh(loops=loops, block=False) 

def main():
    bass(loops=loops, block=False)
    lead(loops=loops, block=False)
    kick(loops=loops, block=False)
    hh(loops=loops, block=False)

if __name__ == "__main__":
    # lead(loops=loops, block=False)
    # bass(loops=loops, block=False)
    kick(loops=loops, block=False)
    hh(loops=loops, block=False) 
    # lead.stop()

# bass(loops=loops, block=False)
# kick(loops=loops, block=False)
# hh(loops=loops, block=False)
