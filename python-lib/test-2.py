from midi_daw import *


@play_on("microKORG2:0", channel=Ch1, blocking=True)
def main():
    note("c4", Hn(1), vel=100, block=False)
    note("g4", Hn(1), vel=100, block=True)


@play_on("SP-404MKII:0", channel=Ch1, blocking=True)
def main_2():
    note("d#3", Qn(1), vel=100, block=True)
    rest(Hn(1))
    note("b2", Qn(1), vel=100, block=True)


main_2()
