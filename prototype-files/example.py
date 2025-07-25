"""
example.py

an example file that shows how the live coding portion of this code base should be used.

NOTE: all midi control functions should send singles to the server over a unix socket.


By: Calacuda | MIT License | Epoch: Jul 22, 2025
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
    """plays a note"""
    midi_out(f"playing note: {note}")


def cc(cc: int, value: float):
    """sends a cc value"""
    pass


def set_tempo(tempo: int):
    """sets the tempo on the server"""
    pass


def wait_for(event: str):
    """used to wait or block on event"""
    pass


def trigger(event: str):
    """used to trigger an event"""
    pass


def lfo(
    freq: float,  # make this a class or enum that can be per beats, per quarter/eighth/sixteeth/etc note, or based on seconds
    lfo_type: str,
    callback: callable,
    one_shot: bool = True,
    bipolar: bool = False,
    hifi_update: bool = False,
) -> str:
    """
    set up a LFO automation

    params:
        freq => the freequency of the lfo oscilation
        lfo_type => what kind of lfo is this? Options
            - from-wav => import an LFO from a wav file.
            - sin => sttandard sin wave.
            - triangle => standard triangle wave.
            - saw-up => standard saw tooth wave going up.
            - saw-down => standard saw tooth wave going down.
            - anti-log => anti-log tapering triable wave.
            - anti-log-up => anti-log taper going up.
            - anti-log-down => anti-log taper going down.
        callable => a callback callable that will be called on every update of the lfo.
        one_shot => if true the lfo will run to compleation only once, else it will run indefinately.
        bipolar => if true the lfo will go above and bellow zero else it will stay positive.
        hifi_update => should this update on an audio sample rate. if false it updated every beat (24-beat per quarter note by default)

    returns: the LFO name to use to turn it off.
    """
    return "LFO_NAME"


def lfo_off(lfo_name: str):
    """turns off an LFO"""
    pass


def adsr(
    atk: float,
    decay: float,
    sustain: float,
    release: float,
    callback: callable,
    hifi_update: bool = False,
) -> str:
    """
    set up an ADSR Envelope automation

    params:
        atk: adsr attack
        decay: adsr deccay
        sustain: adsr sustain
        release: adsr release
        callback: a callback callable that will be called on every update of the adsr
        hifi_update => should this update on an audio sample rate. if false it updated every beat (24-beat per quarter note by default)

    returns: the name of the adsr, used to stop it
    """
    return "ADSR_NAME"


def adsr_off(adsr_name: str):
    """turns off an adsr"""
    pass


def play_on_dev(midi_output, channel="0", blocking=True):
    """

    params:
        blocking => should it be started in a sub thread
    """

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
