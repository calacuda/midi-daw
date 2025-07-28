from midi_daw import *


@play_on("MIDI-OUT", channel=Ch1, blocking=True)
def main_1():
    note("c#4", Qn(1), vel=80, block=False)
    note("c#3", Qn(1), vel=80, block=True)


@play_on("OTHER-OUTPUT", channel=Ch15, blocking=True)
def main_2():
    note("c#3", Qn(1), vel=75, block=True)


def main_3():
    note("c#5", Qn(1), vel=64, block=True)


print("main_1")
main_1()
print()
print("main_1.2")
main_1()
print()
print("main_2")
main_2()
print()
print("main_3")
main_3()
print()
print("main_3.2")
set_midi_chan(5)
main_3()
print()
main_2.compile()
print()
main_2()
