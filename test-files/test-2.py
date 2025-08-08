from midi_daw import *

devs = get_devs()
print(devs)


def start_event():
    wait_for("1")


@play_on("TD-3-MO:0", channel=Ch4, block=False, loop=-1)
def lead():
    note("b2", qn(), vel=80)
    note("a#2", sn(), vel=80)
    note("f#2", en(), vel=80)
    note("C#3", qn(), vel=80)
    note("f#2", en(), vel=80)
    note("C#3", en(), vel=80)
    note("f#3", sn(), vel=80)


@play_on("TD-3-MO:0", channel=Ch4, block=False, loop=-1)
@lfo("test-wavetable.wav", 10.5)
def pitch_wobble(lfo):
    # print(lfo)
    bend_amt = lfo * 0.75
    # do pitch bend
    # print(bend_amt)
    pitch_bend(bend_amt)


lead()
# pitch_wobble()

# rest(wn())
# print("stopping")
# pitch_wobble.stop()
