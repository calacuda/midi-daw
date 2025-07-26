from midi_daw import *


@play_on("MIDI-OUT", channel="0", blocking=True)
def main_1():
    note("c#4", "qn", vel=64, block=False)
    note("c#3", "qn", vel=64, block=True)


@play_on("OTHER-OUTPUT", channel="0", blocking=True)
def main_2():
    note("c#3", "qn", vel=64, block=True)


def main_3():
    note("c#5", "qn", vel=64, block=True)


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
main_2.compile()
