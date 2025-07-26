"""
main.py

a library that allows for live coding.

NOTE: all midi control functions should send singles to the server over a unix socket.


By: Calacuda | MIT License | Epoch: Jul 25, 2025
"""

import threading
from copy import copy
from functools import partial

MIDI_DEV = "MIDI THRU"
MIDI_CHANNEL = "0"


# Start threads for each link
threads = []
running_funcs = {}


def set_midi_output(dev: str, channel=None):
    """sets the midi output device and channel"""
    global MIDI_DEV
    global MIDI_CHANNEL

    if channel is not None:
        MIDI_CHANNEL = channel

    MIDI_DEV = dev


def set_midi_chan(channel):
    """sets the midi channel"""
    global MIDI_CHANNEL

    MIDI_CHANNEL = channel


def clear_dead_threads():
    global threads
    global running_funcs

    threads = [thread for thread in threads if thread.is_alive()]
    running_funcs = {
        name: thread for (name, thread) in running_funcs.items() if thread.is_alive()
    }


def _do_midi_out(midi_dev: str, midi_chan: str, midi_cmd):
    print(f"{midi_cmd} => {midi_dev}:{midi_chan}")


def _midi_out(midi_dev: str, midi_chan: str, midi_cmd, block: bool = True):
    """sends midi to the rust backend"""
    global threads

    if not block:
        # send requests in a sub thread
        t = threading.Thread(
            target=_do_midi_out,
            args=(
                midi_dev,
                midi_chan,
                # MIDI_DEV,
                # MIDI_CHANNEL,
                midi_cmd,
            ),
        )
        t.start()
        threads.append(t)
        clear_dead_threads()

        return None
    else:
        # send request to rust back end
        # return _do_midi_out(MIDI_DEV, MIDI_CHANNEL, midi_cmd)
        return _do_midi_out(midi_dev, midi_chan, midi_cmd)


# midi_out = partial(_midi_out, MIDI_DEV, MIDI_CHANNEL)
def midi_out(midi_cmd, block: bool = True):
    print("DEFAULT MIDI OUT CALLED")
    _midi_out(MIDI_DEV, MIDI_CHANNEL, midi_cmd, block)


def note(note: str, duration: str, vel=80, block=True, midi_out=midi_out):
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


def play_on(midi_output: str, channel="0", blocking=False):
    """
    params:
        blocking => should it be started in a sub thread
    """

    class Decorator:
        def __init__(self, func):
            new_midi_out = partial(_midi_out, midi_output, channel)
            new_note = partial(note, midi_out=new_midi_out)
            self.midi_file = ""
            self.midi_dev = midi_output
            self.channel = channel
            self.func = func
            self.blocking = blocking
            self.name = f"{func.__name__}:{midi_output}:{channel}"
            self.api = {"note": new_note}

        def __call__(self, *args, **kwargs):
            global running_funcs

            self.midi_file = ""
            print('running "midi file" on server')

            old = copy(self.func.__globals__)
            self.func.__globals__.update(self.api)

            # return result
            if not self.blocking:
                t = threading.Thread(target=self.func, args=args, kwargs=kwargs)
                t.start()
                print(f"running function => {self.name}")
                running_funcs[self.name] = t
                clear_dead_threads()
                self.func.__globals__.update(old)

                return None
            else:
                result = self.func(*args, **kwargs)
                self.func.__globals__.update(old)

                return result

        def compile(self):
            """will compile the code and send it to the server for play back"""
            print("compiling")

    return Decorator


# @play_on_dev("MIDI-OUT", channel="0")
# def main_1():
#     note("c#4", "qn", vel=64, block=False)
#     note("c#3", "qn", vel=64, block=True)
#
#
# @play_on_dev("MIDI-OUT", channel="0")
# def main_2():
#     note("c#3", "qn", vel=64, block=True)
#
#
# print("main_1")
# main_1()
# print("main_1.2")
# main_1()
# print("main_2")
# main_2()


# Idea 4


# in the midi_out lib thered be a global that would be mutated by "set_midi_output" and would be used by the "note function"
# from midi_out import note, set_midi_output
#
# set_midi_output("MIDI_DEVICE", "0")
#
#
# def main():
#     note("c#4", "qn", vel=64, block=False)
