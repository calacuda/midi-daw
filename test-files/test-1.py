from midi_daw import *

devs = get_devs()
print(devs)


def chord_prog_1():
    note(["b3", "d4", "f#4", "a4", "c#5"], Qn(1), vel=80, block=True)
    note(["a#3", "C#4", "D4", "f4", "g#4"], Sn(1), vel=80, block=True)
    note(["f#3", "a#4", "c#4", "d#4", "e4"], En(1), vel=80, block=True)
    note(["C#4", "D#4", "e4", "g#4", "b4"], Qn(1), vel=80, block=True)
    note(["f#3", "g#3", "A#3", "b3", "c#4", "d#4"], En(1), vel=80, block=True)


@play_on("microKORG2:0", channel=Ch1, blocking=False)
def chords():
    while True:
        note(["b3", "d4", "f#4", "a4", "c#5"][0], Qn(1), vel=80, block=True)
        note(["a#3", "C#4", "D4", "f4", "g#4"][0], Sn(1), vel=80, block=True)
        note(["f#3", "a#4", "c#4", "d#4", "e4"][0], En(1), vel=80, block=True)
        note(["C#4", "D#4", "e4", "g#4", "b4"][0], Qn(1), vel=80, block=True)
        note(["f#3", "g#3", "A#3", "b3", "c#4", "d#4"][0], En(1), vel=80, block=True)


def sp(pad_n: int) -> int:
    if pad_n <= 4:
        return 47 + pad_n
    elif pad_n <= 8:
        return 43 + (pad_n - 4)
    elif pad_n <= 12:
        return 39 + (pad_n - 8)
    elif pad_n <= 16:
        return 35 + (pad_n - 12)


def do_drums():
    note([sp(4), sp(1)], Sn(1), vel=100, block=True)
    note(sp(1), Sn(1), vel=100, block=True)
    note([sp(7), sp(1)], Sn(1), vel=100, block=True)
    note(sp(1), Sn(1), vel=100, block=True)
    note([sp(4), sp(1)], Sn(1), vel=100, block=True)
    note(sp(1), Sn(1), vel=100, block=True)
    note([sp(3), sp(1)], Sn(1), vel=100, block=True)
    note(sp(1), Sn(1), vel=100, block=True)


@play_on("SP-404MKII:0", channel=Ch1, blocking=False)
def drums():
    # 36 - 52
    # print(sp(4), 52)
    while True:
        do_drums()


chords()
drums()
