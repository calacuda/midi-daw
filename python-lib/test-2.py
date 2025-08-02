from midi_daw import *


@play_on("microKORG2:microKORG2 _ SOUND 20:0", channel=Ch1, blocking=True)
def main():
    note("c4", Hn(1), vel=100, block=False)
    note("g4", Hn(1), vel=100, block=True)


main()
