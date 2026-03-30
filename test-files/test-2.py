import struct

from midi_daw import *


new_dev("VITAL")
devs = get_devs()
print(devs)

counter = 0

# def start_event():
#     wait_for("1")


@play_on("TD-3-MO:0", channel=Ch1, block=False, loop=-1)
def lead():
    panic(lead.new_midi_out)
    note("b2", qn(), vel=80)
    note("a#2", sn(), vel=80)
    note("f#2", en(), vel=80)
    note("C#3", qn(), vel=80)
    note("f#2", en(), vel=80)
    note("C#3", en(), vel=80)
    note("f#3", sn(), vel=80)


@play_on("vital", channel=Ch1, block=False, loop=True)
@lfo("sin", 6.5)
def pitch_wobble(lfo_amt):
    global counter
    # print(lfo)
    # bend_amt = lfo * 0.75
    # do pitch bend
    # print(bend_amt)
    # pitch_bend(bend_amt)
    # print(lfo_amt)
    # pass
    # if lfo_amt:
    #     print(lfo_amt)

    counter += 1
    
    if not (counter % 24):
        # lfo_amt = lfo_amt * 0.5 + 0.5
        # print(lfo_amt)
        bend_amt = lfo_amt * 0.75
        print(f"{lfo_amt} => {bend_amt}")
        # print(f"bend amt {bend_amt}")
        pitch_bend(bend_amt)
        # cc(1, int(lfo_amt * 126))
        note("a#4", en(), vel=80)


# lead()
pitch_wobble()

# rest(wn())
# print("stopping")
# pitch_wobble.stop()
