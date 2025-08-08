from time import sleep

from midi_daw import *

devs = get_devs()
print(devs)


def start_event():
    wait_for("1")


def chord_prog_1():
    note(["b3", "d4", "f#4", "a4", "c#5"], qn(), vel=80)
    note(["a#3", "C#4", "D4", "f4", "g#4"], sn(), vel=80)
    note(["f#3", "a#4", "c#4", "d#4", "e4"], en(), vel=80)
    note(["C#4", "D#4", "e4", "g#4", "b4"], qn(), vel=80)
    note(["f#3", "g#3", "A#3", "b3", "c#4", "d#4"], en(), vel=80)


@play_on("mono:1", channel=Ch1, block=False, loop=-1, setup=start_event)
# @play_on("monologue:1", channel=Ch1, block=False, loop=-1, setup=start_event)
def lead():
    note("b5", qn(), vel=80)
    note("a#5", sn(), vel=80)
    note("f#5", en(), vel=80)
    note("C#6", qn(), vel=80)
    note("f#5", en(), vel=80)
    note("C#6", en(), vel=80)
    note("f#6", sn(), vel=80)
    # rest(sn())


@play_on("td3", channel=Ch4, block=False, loop=-1, setup=start_event)
# @play_on("TD-3-MO:0", channel=Ch4, block=False, loop=-1)
def bass():
    note("b2", qn(), vel=100)
    note("a#2", sn(), vel=80)
    note("f#2", en(), vel=100)
    note("c#3", qn(), vel=80)
    note("f#2", en(), vel=100)
    note("c#2", en(), vel=80)
    note("f#2", sn(), vel=80)
    # rest(sn())


def wait_time():
    import time

    time.sleep(0.25)


@play_on("microKORG2:0", channel=Ch1, block=False, loop=-1, setup=start_event)
def chords():
    note(["b3", "d4", "f#4", "a4", "c#5"], qn(), vel=80)
    note(["a#3", "C#4", "D4", "f4", "g#4"], sn(), vel=80)
    note(["f#3", "a#4", "c#4", "d#4", "e4"], sn(), vel=80)
    rest(sn())
    note(["C#4", "D#4", "e4", "g#4", "b4"], qn(), vel=80)
    rest(sn(4))
    note(["f#3", "g#3", "A#3", "b3", "c#4", "d#4"], sn(1), vel=80)
    # note(["C#4", "D#4", "e4", "g#4", "b4"], en(), vel=80, block=True)
    # note(["a#3", "C#4", "D4", "f4", "g#4"], sn(), vel=80, block=True)


def sp(pad_n: int) -> int:
    if pad_n <= 4:
        return 47 + pad_n
    elif pad_n <= 8:
        return 43 + (pad_n - 4)
    elif pad_n <= 12:
        return 39 + (pad_n - 8)
    elif pad_n <= 16:
        return 35 + (pad_n - 12)


def do_drums(pad):
    note([sp(4), sp(1)], sn(), vel=100)
    note(sp(1), sn(1), vel=100)
    note(
        [sp(7), sp(1)],
        sn(),
        vel=127,
    )
    note(sp(1), sn(1), vel=100)
    note([sp(4), sp(1)], sn(), vel=100)
    note(sp(1), sn(1), vel=100)
    note([sp(3)], sn(), vel=100)
    note(sp(pad), sn(1), vel=100)


# @play_on("SP-404MKII:0", channel=Ch1, block=False, loop=-1, setup=start_event)
@play_on("sp404", channel=Ch1, block=False, loop=-1, setup=start_event)
# @play_on("SP-404MKII:0", channel=Ch1, block=False, loop=-1)
def drums():
    # 36 - 52
    do_drums(1)
    do_drums(9)
    # panic()
    # rest(sn())


# chords()
drums()
bass()
lead()

sleep(5)

# drums.stop()
# chords.stop()


# @play_on("TD-3-MO:0", channel=Ch4, block=False, loop=False)
# def stop_test():
#     note("b2", wn(2), vel=80)


# sleep(0.5)
# drums.stop()
# stop_test()
# rest(en())
# stop_test.stop()

# print("stopped")
