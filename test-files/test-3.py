from midi_daw import *

dev_name = "VITAL"

new_dev(dev_name)

devs = get_devs()
print(devs)


set_midi_output(dev_name)


# @play_on("TD-3-MO:0", channel=Ch4, block=False, loop=-1)
def lead():
    panic()
    while True:
        note("b2", qn(), vel=80)
        note("a#2", sn(), vel=80)
        note("f#2", en(), vel=80)
        note("C#3", qn(), vel=80)
        note("f#2", en(), vel=80)
        note("C#3", en(), vel=80)
        note("f#3", sn(), vel=80)


lead()
