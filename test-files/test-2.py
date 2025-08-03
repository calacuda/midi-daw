from midi_daw import *

devs = get_devs()
print(devs)


@play_on("TD-3-MO:0", channel=Ch4, blocking=False, loop=-1)
def lead():
    note("f#3", qn(), vel=80, block=True)
    note("D3", sn(), vel=80, block=True)
    note("c#3", en(), vel=80, block=True)
    note("e3", qn(), vel=80, block=True)
    note("A#2", sn(5), vel=80, block=True)
    note("e3", en(), vel=80, block=True)
    note("D3", sn(), vel=80, block=True)


lead()
