"""
main.py

a library that allows for live coding.

NOTE: all midi control functions should send singles to the server over a unix socket.


By: Calacuda | MIT License | Epoch: Jul 25, 2025
"""

import logging
import threading
from copy import copy
from functools import partial
from multiprocessing import Process

import requests
import requests_unixsocket
from midi_daw_types import (UDS_SERVER_PATH, MidiChannel, MidiMsg, MidiReqBody,
                            MidiTarget, NoteLen, note_from_str)

requests_unixsocket.monkeypatch()
log = logging.getLogger(__name__)
logging.basicConfig(level=logging.INFO)

MIDI_TARGET = MidiTarget()

# Start threads for each link
threads = []
processes = []
running_funcs = {}


def mk_channel(channel):
    ch = None

    if channel is None:
        ch = None
    elif isinstance(channel, MidiChannel):
        ch = channel
    elif isinstance(channel, str):
        ch = MidiChannel.from_hex(channel)
    elif isinstance(channel, int):
        ch = MidiChannel.from_int(channel)
    else:
        ch = MidiChannel()

    return ch


def set_midi_output(dev: str, channel=None):
    """sets the midi output device and channel"""
    global MIDI_TARGET

    channel = mk_channel(channel)

    if channel is not None:
        print("channel is", channel)
        MIDI_TARGET.ch = channel

    MIDI_TARGET.name = dev


def set_midi_chan(channel):
    """sets the midi channel"""
    global MIDI_TARGET

    MIDI_TARGET.ch = mk_channel(channel)


def clear_dead_threads():
    global threads
    global running_funcs

    threads = [thread for thread in threads if thread.is_alive()]
    running_funcs = {
        name: thread for (name, thread) in running_funcs.items() if thread.is_alive()
    }


def post(data, path):
    socket = UDS_SERVER_PATH.replace("/", "%2F")
    headers = {"Content-Type": "application/json"}
    res = requests.post(
        f"http+unix://{socket}/{path}",
        data=data,
        headers=headers,
    )

    if res.status_code != 200:
        log.warning(f"{res.text}")


def get(path):
    socket = UDS_SERVER_PATH.replace("/", "%2F")
    # headers = {"Content-Type": "application/json"}
    res = requests.get(
        f"http+unix://{socket}/{path}",
        # data=data,
        # headers=headers,
    )

    if res.status_code != 200:
        log.warning(f"{res.text}")

    return res.json()


def _do_midi_out(midi_target: MidiTarget, midi_cmd: MidiMsg):
    post(MidiReqBody(midi_target.name, midi_target.ch, midi_cmd).json(), "midi")


def _midi_out(midi_target: MidiTarget, midi_cmd: MidiMsg, block: bool = True):
    """sends midi to the rust backend"""
    global threads

    if not block:
        # clear_dead_threads()
        # send requests in a sub thread
        t = threading.Thread(
            # t = Process(
            target=_do_midi_out,
            args=(
                midi_target,
                midi_cmd,
            ),
        )
        t.start()
        threads.append(t)

        return None
    else:
        # send request to rust back end
        return _do_midi_out(midi_target, midi_cmd)


# midi_out = partial(_midi_out, MIDI_DEV, MIDI_CHANNEL)
def midi_out(midi_cmd: MidiMsg, block: bool = True):
    # print("DEFAULT MIDI OUT CALLED")
    _midi_out(MIDI_TARGET, midi_cmd, block)


def note(note, duration: NoteLen, vel=80, block: bool = True, midi_out=midi_out):
    """plays a note"""
    # add int, and lists types as notes

    def mk_cmd(note):
        midi_cmd = None

        if isinstance(note, int):
            midi_cmd = MidiMsg.PlayNote(note, vel, duration)
        elif isinstance(note, str):
            midi_cmd = MidiMsg.PlayNote(note_from_str(note), vel, duration)

        return midi_cmd

    def send_midi_cmd(cmd, block=block):
        if cmd is not None:
            midi_out(cmd, block=block)

    if isinstance(note, list) and not isinstance(note, str):
        for n in note[:-1]:
            midi_cmd = mk_cmd(n)
            send_midi_cmd(midi_cmd, block=False)

        if len(note) > 1:
            midi_cmd = mk_cmd(note[-1])
            send_midi_cmd(midi_cmd)
    else:
        midi_cmd = mk_cmd(note)
        send_midi_cmd(midi_cmd)


def rest(duration: NoteLen):
    """musical rest"""
    post(duration.json(), "rest")


def cc(cc: int, value: float, midi_out=midi_out):
    """sends a cc value"""
    midi_cmd = MidiMsg.CC(cc, value)
    midi_out(midi_cmd)


def set_tempo(tempo: int):
    """sets the tempo on the server"""
    post(float(tempo), "tempo")


def get_tempo() -> float:
    return 0.0


def get_devs() -> list[str]:
    return get("midi")


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
    # midi_out=midi_out,
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
    # midi_out=midi_out,
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


def all_off(midi_output: str, channel=MidiChannel.Ch1):
    """stops all playing notes on device: midi_output on channel: channel"""
    pass


def play_on(midi_output: str, channel=MidiChannel.Ch1, loop=0, blocking=False):
    """
    params:
        blocking => should it be started in a sub thread
    """

    class Decorator:
        def __init__(self, func):

            self.midi_file = ""
            self.midi_dev = midi_output
            self.channel = mk_channel(channel)
            self.func = func
            self.blocking = blocking
            self.name = f"{func.__name__}:{midi_output}:{self.channel}"
            self.midi_target = MidiTarget()
            self.midi_target.name = self.midi_dev
            self.midi_target.ch = self.channel
            new_midi_out = partial(_midi_out, self.midi_target)
            new_note = partial(note, midi_out=new_midi_out)
            new_cc = partial(cc, midi_out=new_midi_out)
            # new_rest = partial(rest)
            self.api = {"note": new_note, "cc": new_cc}
            # print(dir(self.func))
            self.should_loop = loop != 0
            self.loop_number = -1 if isinstance(loop, bool) and loop else loop

        def __call__(self, *args, **kwargs):
            global running_funcs

            self.midi_file = ""
            # print('running "midi file" on server')

            old = copy(self.func.__globals__)
            self.func.__globals__.update(self.api)

            # return result
            if not self.blocking:
                clear_dead_threads()
                # t = threading.Thread(target=self.func, args=args, kwargs=kwargs)
                f = self.loop_f if self.should_loop else self.func
                t = Process(target=f, args=args, kwargs=kwargs)
                t.start()
                self.func.__globals__.update(old)
                # print(f"running function => {self.name}")
                running_funcs[self.name] = t
                # clear_dead_threads()

                return None
            else:
                result = self.func(*args, **kwargs)
                self.func.__globals__.update(old)

                return result

        def loop_f(self, *args, **kwargs):
            # while True:
            if self.loop_number < 0:
                while True:
                    self.func(*args, *kwargs)
            else:
                for _ in range(self.loop_number):
                    self.func(*args, *kwargs)

        def compile(self):
            """will compile the code and send it to the server for play back"""
            print("compiling")

    return Decorator
