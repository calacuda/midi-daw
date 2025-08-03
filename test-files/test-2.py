from midi_daw import *

devs = get_devs()
print(devs)


def start_event():
    wait_for("1")


@play_on("monologue:1", channel=Ch1, block=False, loop=-1, setup=start_event)
def lead():
    note("b5", qn(), vel=80)
    note("a#5", sn(), vel=80)
    note("f#5", en(), vel=80)
    note("C#6", qn(), vel=80)
    note("f#5", en(), vel=80)
    note("C#6", en(), vel=80)
    note("f#5", sn(), vel=80)


lead()
