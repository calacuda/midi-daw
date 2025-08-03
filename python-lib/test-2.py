from midi_daw import *

devs = get_devs()
print(devs)


@play_on("microKORG2:0", channel=Ch1, blocking=False)
def main():
    # note("c4", Hn(1), vel=80, block=False)
    # note("e4", Hn(1), vel=80, block=False)
    # note("g4", Hn(1), vel=80, block=True)
    note(["c4", "e4", "g4"], Hn(1), vel=80, block=True)


@play_on("SP-404MKII:0", channel=Ch1, blocking=False)
def main_2():
    # 36 - 52
    note("d#3", Sn(1), vel=100, block=True)
    rest(Sn(1))
    note("b2", Sn(1), vel=100, block=True)
    rest(Sn(1))
    note([42, 40, 46], Sn(1), vel=100, block=True)
    # note("b2", Sn(1), vel=100, block=True)
    # for i in range(16):
    #     note(36 + i, Sn(2), vel=100, block=True)


main()
main_2()
