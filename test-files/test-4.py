from midi_daw import *


new_dev("synth-1")
new_dev("synth-2")
devs = get_devs()
print(devs)

# def start_event():
#     wait_for("1")

is_set = False


@play_on("synth-1", channel=Ch1, block=False, loop=-1)
def lead(synth):
    # print(synth.midi_target.name)
    # panic(lead.new_midi_out)
    # note("b2", qn(), vel=80)
    # note("a#2", sn(), vel=80)
    # note("f#2", en(), vel=80)
    # note("C#3", qn(), vel=80)
    # note("f#2", en(), vel=80)
    # note("C#3", en(), vel=80)
    # note("f#3", sn(), vel=80)
    synth.cc(0, 128//2)
    # print(lead.__globals__.get("is_set"))
    # print("lead", lead.__globals__.get("MIDI_TARGET"))


@play_on("synth-2", channel=Ch1, block=False, loop=-1)
def kick(synth):
    # print("kick")
    # print(synth.midi_target.name)
    # print(lead.__globals__.get("is_set"))
    synth.note(33, sn(), vel=80)
    # print(kick.__globals__)
    # print(lead.__globals__.get("is_set"))
    # print("played")
    rest(sn(3))
    # print("rested")
    # print(lead.__globals__.get("is_set"))
    # print("kick", kick.__globals__.get("MIDI_TARGET"))


# print("outside 1", lead.__globals__.get("MIDI_TARGET"))
lead()
kick()
# print("outside 1", lead.__globals__.get("MIDI_TARGET"))

# import time


# time.sleep(30)

# rest(wn())
# print("stopping")
# pitch_wobble.stop()
