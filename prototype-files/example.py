"""
all midi control functions should send singles to teh server over a unix socket.

"""

# idea 1


# send = Player(
#     "MIDI-OUT",
#     channel="0",
# )
#
#
# def main():
#     send.note("c#4", "qn", vel=64, b=False)


# idea 2


# def midi_out(midi_output, midi_channel):
#     def note(
#         note: str,
#         duration: str,
#         vel=80,
#         block=True,
#     ):
#         print(f"playing note: {note} on device: {midi_output} channel: {midi_channel}")
#
#     def decorator(func):
#         func.__globals__.update({"note": note})
#
#         def wrapper(*args, **kwargs):
#             return func(*args, **kwargs)
#
#         return wrapper
#
#     return decorator
#
#
# @midi_out("MIDI-OUT", "0")
# def main():
#     note("c#4", "qn", vel=64, block=False)
#
#
# main()


#
# idea 3
#
# this would "compile" to a midi file that could then be played
#
# can compile using decorators but also not if using desiered
#


COMPILE = False


def midi_out(*args, block: bool = True):
    if not block:
        print("sending midi to server directly, on a secondary thread.")
    else:
        print("sending midi to server directly.")


def note(
    note: str,
    duration: str,
    vel=80,
    block=True,
):
    midi_out(f"playing note: {note}")


def play_on_dev(midi_output, channel="0"):
    class Decorator:
        def __init__(self, func):
            global COMPILE
            COMPILE = True

            self.midi_file = ""
            self.func = func

        def __call__(self, *args, **kwargs):
            self.midi_file = ""
            self.func.__globals__.update({"midi_out": self.midi_out})
            # print("calling")
            result = self.func(*args, **kwargs)
            print('running "midi" file on server')
            print(self.midi_file)

            return result

        def midi_out(self, *args, block: bool = False):
            """append to a midi file that will get played when the function is called"""
            self.midi_file += "" + " ".join(str(arg) for arg in args)
            self.midi_file += "\n"

    return Decorator


@play_on_dev("MIDI-OUT", channel="0")
def main_1():
    note("c#4", "qn", vel=64, block=False)
    note("c#3", "qn", vel=64, block=True)


@play_on_dev("MIDI-OUT", channel="0")
def main_2():
    note("c#3", "qn", vel=64, block=True)


print("main_1")
main_1()
print("main_1.2")
main_1()
print("main_2")
main_2()


# Idea 4


# in the midi_out lib thered be a global that would be mutated by "set_midi_output" and would be used by the "note function"
# from midi_out import note, set_midi_output
#
# set_midi_output("MIDI_DEVICE", "0")
#
#
# def main():
#     note("c#4", "qn", vel=64, block=False)
