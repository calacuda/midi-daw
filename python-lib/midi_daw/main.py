"""
main.py

a library that allows for live coding.

NOTE: all midi control functions should send singles to the server over a unix socket.


By: Calacuda | MIT License | Epoch: Jul 25, 2025
"""

# import asyncio
import json
import logging
import multiprocessing
import threading
from copy import copy, deepcopy
from functools import partial
from multiprocessing import Manager, Process
from os import path
from dataclasses import dataclass

import requests
import requests_unixsocket
from midi_daw_types import (UDS_SERVER_PATH, Automation, AutomationCommand, 
                            AutomationConf, LfoConfig, MidiChannel, 
                            MidiMsg, MidiReqBody, MidiTarget, NoteLen, 
                            Sequence as BackEndSequence, SetDevBody, AddNoteBody,
                            RmNoteBody, SetChannelBody, note_from_str,)
from thefuzz import process
from websockets.sync.client import unix_connect
import jack
import musical_scales


class WhitelistFilter(logging.Filter):
    """Filters log records based on a whitelist of logger names."""
    def __init__(self, *whitelist):
        self.whitelist = whitelist

    def filter(self, record):
        # Check if the logger name (or starts with) is in the whitelist
        for name in self.whitelist:
            if record.name.startswith(name):
                return True
        return False

requests_unixsocket.monkeypatch()
_filter = WhitelistFilter(__name__, "midi_daw_types", "midi_daw")
log = logging.getLogger(__name__)
logging.basicConfig(level=logging.INFO)
# Add the filter to the default handler
for handler in logging.root.handlers:
    handler.addFilter(_filter)

multiprocessing.set_start_method("fork", force=True)  # or 'spawn'

MIDI_TARGET = MidiTarget()

# Start threads for each link
threads = []
processes = []
running_funcs = {}
AUTOMATION_THREADS = {}


class AutomationWrapper:
    def __init__(self, func, sub_type: str, automation=None, main_type=None):
        global _JACK_CLIENT

        self._func = deepcopy(func)
        self.name = f"{func.__name__}.{main_type}.{sub_type}"
        self.automation = automation
        self.__name__ = self.name
        self.__globals__ = self._func.__globals__
        self._jack_client = jack.Client('midi-daw-live')
        self.input = self._jack_client.inports.register(self.name)
        # _JACK_CLIENT.connect(out_port, self.input.name)
        self.event = threading.Event()

    def __call__(self, *args, **kwargs):
        import time 
        global _JACK_CLIENT
        # global AUTOMATION_THREADS

        # t = threading.Thread(target=self.func, args=args, kwargs=kwargs)
        # f = self.loopLesson 4 - Cable Car_f if self.should_loop else self.func
        # f = self.func
        # t = Process(target=f, args=args, kwargs=kwargs)
        # t.start()
        # # self.func.__globals__.update(old)
        # # print(f"running function => {self.name}")
        # AUTOMATION_THREADS[self.name] = t
        # return self.name

        # stop old automation
        # post(AutomationCommand.Stop(self.name), "automation")
        # return self.func(*args, **kwargs)
        self.stop()
        post(AutomationCommand.New(self.name, self.automation).json(), "automation")
        time.sleep(.25)
        self._jack_client = jack.Client('midi-daw-live')
        self.input = self._jack_client.inports.register(self.name)
        self._jack_client.set_process_callback(self.func)
        # print("callback set")
        out_ports = [port.name for port in self._jack_client.get_ports(is_audio=True, is_output=True, is_physical=False)]
        # fuzzy find for "midi-daw:{name}" and connect to it
        # print(f"found output_ports {out_ports}")
        out_port = process.extractOne(f"midi-daw:{self.name}", out_ports)[0]
        # print(f"found output_port {out_port} it matches {self.name}")
        self._jack_client.activate()
        # print("activated")
        self.input.connect(out_port)
        # print(f"connected, {self.input.connections}")
        self.event.wait()

    def func(self, frames):
        # print("foo")
        # auto_val = self.automation.step()
        # auto_val = with self.

        # print("auto_val", auto_val)
        # args = (auto_val,) + args
        # print(frames)
        # print(type(frames))
        # print(dir(frames))
        for sample in self.input.get_array():
            self._func(sample)

    def stop(self):
        """stops the running automation thread"""
        # global AUTOMATION_THREADS
        #
        # proc = AUTOMATION_THREADS.get(self.name)
        #
        # if proc is not None:
        #     proc.kill()
        self.event.set()
        post(AutomationCommand.Stop(self.name).json(), "automation")
        # pass


    # def init_automation(self):
    #     """initializes the automation to initial state"""
    #     if self.automation is not None:
    #         self.automation.init()

    # def reset_automation(self):
    #     """ressets the automation"""
    #     if self.automation is not None:
    #         self.automation.reset()

    def get_name(self):
        """returns the threads name"""
        return self.name


@dataclass
class Scale:
    root: str
    scale: str
    octave: int = 4
    # def __init__(self):
    #     self.scale


def set_log_level(level):
    logging.getLogger().setLevel(level)


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

    if channel is not None:
        channel = mk_channel(channel)
        log.debug("channel is", channel)
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


def get(path, convert: bool = True, **query):
    socket = UDS_SERVER_PATH.replace("/", "%2F")
    # headers = {"Content-Type": "application/json"}
    res = requests.get(
        f"http+unix://{socket}/{path}",
        # data=data,
        # headers=headers,
        params=dict(query) if query is not None else None
    )

    if res.status_code != 200:
        log.warning(f"{res.text}")

    if convert:
        return res.json()
    else:
        return res.text


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


def mk_midi_note(note, scale: Scale | None = None) -> int:
    midi_note = -1

    match note:
        case str():
            # midi_note = note_from_str(note)
            if scale is None:
                # midi_note = musical_scales.Note(note).enharmonic
                midi_note = note_from_str(note)
            elif note.isnumeric():
                m_scale = musical_scales.scale(scale.root, scale.scale)
                print(m_scale)
                # print(f"{int(note)} // {len(m_scale)} = {int(note) // len(m_scale)}")

                octave = scale.octave + (int(note) - 1) // len(m_scale)
                note = m_scale[(int(note) - 1) % len(m_scale)]
                # print(f"note 1: {note}")

                if note.midi[-2:0].isnumeric():
                    octave += int(note.midi[-2:]) - 3
                    note = note.midi[0:-2] + str(octave)
                else:
                    octave += (int(note.midi[-1]) - 3)
                    note = note.midi[0:-1] + str(octave)
                print(f"note 2: {note}")
                midi_note = note_from_str(note)
            # midi_cmd = midi_cmd(midi_note)
            # send_midi_cmd(midi_cmd)
        case int():
            # midi_cmd = midi_cmd(notes)
            # send_midi_cmd(midi_cmd)
            midi_note = note
        # case list():
        #     for n in notes[:-1]:
        #         note(n, duration, vel, False, midi_out)
        #
        #     note(notes[-1], duration, vel, block, midi_out)

    return midi_note



def note(
    notes,
    duration: NoteLen = NoteLen.Sn(1),
    vel=80,
    block: bool = True,
    midi_out=midi_out,
):
    """plays a note"""
    midi_cmd = None

    if duration:
        # midi_cmd = MidiMsg.PlayNote
        def midi_cmd(note):
            return MidiMsg.PlayNote(note, vel, duration)

    else:
        midi_cmd = MidiMsg.StopNote

    def send_midi_cmd(cmd, block=block):
        if cmd is not None:
            # print("note => ", cmd)
            midi_out(cmd, block=block)

    match notes:
        case str():
            midi_note = note_from_str(notes)
            midi_cmd = midi_cmd(midi_note)
            send_midi_cmd(midi_cmd)
        case int():
            midi_cmd = midi_cmd(notes)
            send_midi_cmd(midi_cmd)
        case list():
            for n in notes[:-1]:
                note(n, duration, vel, False, midi_out)

            note(notes[-1], duration, vel, block, midi_out)


def pitch_bend(amt, block: bool = True, midi_out=midi_out):
    abs_val = abs((amt + 1.0) * (16383 / 2))
    amt = int(abs_val)
    cmd = MidiMsg.PitchBend(amt)
    midi_out(cmd, block=block)


def stop_notes(notes, midi_out=midi_out):
    for n in notes:
        note(n, None, midi_out=midi_out)


def panic(midi_out=midi_out):
    """stops all playing notes on device: midi_output on channel: channel"""
    note(list(range(128)), None, block=True, midi_out=midi_out)


def rest(duration: NoteLen):
    """musical rest"""
    post(duration.json(), "rest")


def cc(cc: int, value: float, midi_out=midi_out):
    """sends a cc value"""
    midi_cmd = MidiMsg.CC(cc, value)
    midi_out(midi_cmd)


def set_tempo(tempo: float):
    """sets the tempo on the server"""
    post(str(float(tempo)), "tempo")


def get_tempo() -> float:
    return get("tempo")


def tempo(new_tempo: float | None = None) -> float:
    if new_tempo is not None:
        set_tempo(new_tempo)

    return get_tempo()


def get_devs() -> list[str]:
    return get("midi")


def find_dev(dev_name) -> str:
    """uses a fuzzy finding algo to get the proper device"""
    choices = get_devs()
    one = process.extractOne(dev_name, choices)

    if one is None:
        return MIDI_TARGET
    else:
        return one[0]


def wait_for(func):
    """used to wait or block on event"""
    unix_socket_path = UDS_SERVER_PATH
    socket = unix_socket_path.replace("/", "%2F")
    uri = f"ws://{socket}/message-bus"

    def check_one(mb_event):
        # return mb_event == event
        return func(mb_event)

    def check_many(mb_event):
        # return mb_event in event
        # TODO: write this
        return False

    check = None

    if hasattr(func, "__contains__") and not isinstance(func, str):
        check = check_many
    else:
        check = check_one

    with unix_connect(path=unix_socket_path, uri=uri) as ws:
        recv = ws.recv()
        recv = recv.replace('"', "")

        while not check(recv):
            recv = ws.recv().decode("ascii")
            # print(f"recv {recv}")
            recv = recv.replace('"', "")

        log.debug("event recved")


def trigger(event: str):
    """used to trigger an event"""
    unix_socket_path = UDS_SERVER_PATH
    socket = unix_socket_path.replace("/", "%2F")
    uri = f"ws://{socket}/message-bus"

    with unix_connect(path=unix_socket_path, uri=uri) as ws:
        ws.send(event)


def lfo(
    lfo_type: str,
    freq: float,  # make this a class or enum that can be per beats, per quarter/eighth/sixteeth/etc note, or based on seconds
    # callback: callable,
    one_shot: bool = False,
    bipolar: bool = True,
    hifi_update: bool = False,
    # midi_out=midi_out,
):
    """
    set up a LFO automation

    params:
        freq => the freequency of the lfo oscilation
        lfo_type => what kind of lfo is this? Options
            - WaveTable => import an LFO from a wav file.
            - sin => sttandard sin wave.
            - triangle => standard triangle wave.
            - saw-up => standard saw tooth wave going up.
            - saw-down => standard saw tooth wave going down.
            - anti-log => anti-log tapering triable wave.
            - anti-log-up => anti-log saw wave shaped taper going up.
            - anti-log-down => anti-log saw wave shaped taper going down.
        # callable => a callback callable that will be called on every update of the lfo.
        one_shot => if true the lfo will run to compleation only once, else it will run indefinately.
        bipolar => if true the lfo will go above and bellow zero else it will stay positive.
        hifi_update => should this update on an audio sample rate. if false it updated every beat (24-beat per quarter note by default)

    returns: the LFO name to use to turn it off.
    """
    # lfo = []

    # def outer(callback):
    #     def wrapper(*args, **kwargs):
    #         nonlocal lfo
    #
    #         lfo_value = lfo.step()
    #         callback(lfo_value)
    #
    #     return wrapper

    # return "LFO_NAME"
    # return outer
    # TODO: make an LFO automation in rust

    # lfo_type = lfo_type.lower()
    # lfo_types = {
    #     # "wave": None,
    #     "sin": None,
    #     "triangle": None,
    #     "saw-up": None,
    #     "saw-down": None,
    #     "antilog": None,
    #     "antilog-up": None,
    #     "antilog-down": None,
    # }
    # lfo_builder = lfo_types.get(lfo_type)
    #
    # if lfo_builder is not None:
    #     return partial(Automation, automation=lfo_builder)
    # elif lfo_type.endswith(".wav") and path.exists(lfo_type):
    #     # build wave table lfo
    #     conf = LfoConfig.WaveTable(lfo_type, freq)
    #
    #     try:
    #         automation = Automation(AutomationConf.Lfo(conf))
    #     except ValueError as e:
    #         log.error(f"{e}")
    #     else:
    #         # print("returning automation")
    #         return partial(AutomationWrapper, automation=automation, main_type="lfo")
    #
    # elif lfo_type.endswith(".wav") and not path.exists(lfo_type):
    #     print("lfo_type apears to be a file path but it doen't exists.")
    #     return None
    # else:
    #     print("unknown lfo_type")
    #     return None
    lfo_type = lfo_type.lower()
    lfo_types = {
        # "wave": lambda : LfoConfig.WaveTable(),
        "sin": lambda : LfoConfig.Sin(freq, one_shot, bipolar, hifi_update),
        "triangle": None,
        "saw-up": None,
        "saw-down": None,
        # "antilog": None,
        # "antilog-up": None,
        # "antilog-down": None,
    }
    lfo_builder = lfo_types.get(lfo_type)
    
    if lfo_builder is not None:
        automation = AutomationConf.Lfo(lfo_builder())
        return partial(AutomationWrapper, automation=automation, main_type="lfo", sub_type=lfo_type)
    elif lfo_type.endswith(".wav") and path.exists(lfo_type):
        conf = LfoConfig.WaveTable(lfo_type, freq)

        try:
            # automation = Automation(AutomationConf.Lfo(conf))
            automation = AutomationConf.Lfo(conf)
        except ValueError as e:
            log.error(f"{e}")
            return None
        else:
            # print("returning automation")
            return partial(AutomationWrapper, automation=automation, main_type="lfo", sub_type=lfo_type)
            # return AutomationWrapper()
    elif lfo_type.endswith(".wav") and not path.exists(lfo_type):
        print("you are trying to use teh WaveTable LFO but that file path doesn't exists.")
        return None
    else:
        print("unknown lfo_type")
        return None


# def lfo_off(lfo_name: str):
#     """turns off an LFO"""
#     pass


def adsr(
    atk: float,
    decay: float,
    sustain: float,
    release: float,
    callback: callable,
    hifi_update: bool = False,
    # midi_out=midi_out,
):
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
    # return "ADSR_NAME"
    adsr = None
    return partial(Automation, automation=adsr)


# def adsr_off(adsr_name: str):
#     """turns off an adsr"""
#     pass


# def all_off(midi_output: str, channel=MidiChannel.Ch1):
#     pass


def new_dev(dev_name):
    """used to add virtual midi devices"""
    post(f'"{dev_name}"', "new-dev")


def play_on(midi_output: str, channel=MidiChannel.Ch1, loop=0, block=False, setup=None):
    """
    params:
        block => should it be started in a sub thread
    """

    class PlayOn:
        def __init__(self, func):

            self.midi_file = ""
            self.midi_dev = (
                midi_output if midi_out in get_devs() else find_dev(midi_output)
            )
            self.channel = mk_channel(channel)
            self.func = func
            self.is_blocking = block
            self.name = f"{func.__name__}:{midi_output}:{self.channel}"
            self.midi_target = MidiTarget()
            self.midi_target.name = self.midi_dev
            self.midi_target.ch = self.channel
            self.new_midi_out = partial(_midi_out, self.midi_target)
            self.send_note_to = partial(note, midi_out=self.new_midi_out)
            # self.playing_notes = Array(0, 64)
            # self.manager = Manager()
            # self.playing_notes = self.manager.list()
            self.playing_notes = []
            self.new_note = partial(self.note, self.playing_notes)
            self.new_cc = partial(cc, midi_out=self.new_midi_out)
            # new_rest = partial(rest)
            self.new_pitch_bend = partial(pitch_bend, midi_out=self.new_midi_out)
            self.new_panic = partial(panic, midi_out=self.new_midi_out)
            self.api = {
                "note": self.new_note,
                "cc": self.new_cc,
                "panic": self.new_panic,
                "pitch_bend": self.new_pitch_bend,
                # "_midi_out": self.new_midi_out,
                "MIDI_TARGET": self.midi_target,
            }
            # self.api = {"note": self.new_note, "MIDI_TARGET": self.midi_target}
            # print(dir(self.func))
            self.should_loop = loop != 0
            self.loop_number = -1 if isinstance(loop, bool) and loop else loop
            self.setup_f = setup
            self.__name__ = self.name
            self.__globals__ = self.func.__globals__  

        def __call__(self, *args, **kwargs):
            global running_funcs

            self.midi_file = ""
            # print('running "midi file" on server')

            old = copy(self.func.__globals__)
            self.func.__globals__.update(self.api)
            
            if isinstance(self.func, AutomationWrapper):
                self.func.func.__globals__.update(self.api)
                self.func._func.__globals__.update(self.api)
                self.func()
                self.stop()
                # self.func = lambda : None
                # self.func = self.func.func

            # return result
            if not self.is_blocking:
                # clear_dead_threads()
                # t = threading.Thread(target=self.func, args=args, kwargs=kwargs)
                f = self.loop_f if self.should_loop else self.func
                # t = Process(target=f, args=args, kwargs=kwargs)
                t = threading.Thread(target=self.func, args=args, kwargs=kwargs)
                t.start()
                # self.func.__globals__.update(old)
                # print(f"running function => {self.name}")
                running_funcs[self.name] = t
                print(running_funcs)
                clear_dead_threads()

                return self.name
            else:
                result = self.func(*args, **kwargs)
                self.func.__globals__.update(old)
                clear_dead_threads()

                return result


        def note(self, playing_notes, *args, **kwargs):
            # send_note_to = partial(self.note, midi_out=self.new_midi_out)
            try:
                playing_notes.append(args[0])
            except BrokenPipeError:
                pass

            # print(playing_notes)
            self.send_note_to(*args, **kwargs)
            # self.playing_notes = [notes for notes in self.playing_notes if notes is not args[0]]
            try:
                # print("rm")
                playing_notes.remove(args[0])
            except ValueError:
                pass
            except BrokenPipeError:
                pass

        def stop(self):
            """stops the running play back thread"""
            global running_funcs

            proc = running_funcs.get(self.name)

            if proc is not None:
                proc.kill()

            if self.playing_notes:
                stop_notes(self.playing_notes, midi_out=self.new_midi_out)

            if isinstance(self.func, AutomationWrapper):
                # print("func was autowrapper")
                self.func.stop()
            # else:
            #     print(f"type of self.func is: {type(self.func)}")
            # self.new_panic()

            log.info(f"stopped function {self.name}")

        def loop(self, num):
            """sets the number of loops to preform"""
            self.should_loop = loop != 0
            self.loop_number = -1 if isinstance(loop, bool) and loop else loop

        def loop_f(self, *args, **kwargs):
            if self.setup_f is not None:
                self.setup_f()

            if self.loop_number < 0:
                while True:
                    self.func(*args, *kwargs)
            else:
                for _ in range(self.loop_number):
                    self.func(*args, *kwargs)

            self.stop()

        def compile(self):
            """will compile the code and send it to the server for play back"""
            print("compiling")

    return PlayOn


class Sequence:
    default_step_duration = NoteLen.Sn(1)
    default_step_velocity = 88
        
    def __init__(self, name: str, step_duration: NoteLen | None = None, step_velocity: int | None = None, dev: str | None = None, channel: MidiChannel = MidiChannel.Ch1) -> None:
        seq = None
        names = get("sequence/names")
        log.debug(f"found seqeunces :  {names}")
        self.name = name

        if name in names:
            seq = BackEndSequence.from_json(get("sequence", convert=False, sequence=name))
            log.debug(f"found seqeunce :  {seq.name}")
            log.debug([msg for msg in [step for step in seq.steps]])

        if seq is not None:
            log.debug("setting seqeunce to that from the server")
            self.seq = seq
        else:
            self.seq = BackEndSequence(name)
            post(json.dumps(name), "sequence/new")
            self.stop_now()

        self._dev = dev
        self._channel = channel
        self._scale: Scale | None = None

        if self._dev is not None:
            self.seq.midi_dev = self.dev

        if self._channel is not None:
            self.seq.channel = self._channel
            
        self.step_duration = step_duration
        self.step_velocity = step_velocity
 
    def dev(self, value: str | None = None):
        if value is not None:
            value = find_dev(value)
            self.seq.midi_dev = value
            self._dev = value
            post(SetDevBody(self.name, value).json(), "sequence/set-dev")

            return self
        else:
            return self._dev

    def channel(self, value: MidiChannel):
        if value is not None:
            self.seq.channel = value
            self._channel = value
            post(SetChannelBody(self.name, value).json(), "sequence/set-channel")

            return self
        else:
            return self._channel

    def duration(self, value: NoteLen | None = None):
        if value is not None:
            self.default_step_duration = value

            return self
        else:
            return self.default_step_duration


    def scale(self, value: Scale | None = None):
        if value is not None:
            self._scale = value
            
            return self
        else:
            return self._scale

    def step(self, step: int, note: str | int,  duration: NoteLen | None = None, velocity: int | None = None):
        # first non-null value with priority given first to function args, then to object defaults, then class defaults.
        velocity = next((item for item in [velocity, self.step_velocity] if item is not None), self.default_step_velocity)
        duration = next((item for item in [duration, self.step_duration] if item is not None), self.default_step_duration)

        if step >= self.seq.len():
            log.warning(f"attemting to set step: {step}, of sequence of len: {self.seq.len()}. step was too high using modulo to force step to be with in range. will set step: {step % self.seq.len()}")

        note = mk_midi_note(note, scale=self.scale())
        # print(f"{type(note)}: {note}")

        (should_add, old_note) = self.seq.set_note(step % self.seq.len(), note, velocity, duration)

        if old_note is not None:
            log.info("rming an existing note")
            post(RmNoteBody(self.name, step, old_note).json(), "sequence/rm-note")

        if should_add:
            # log.info("adding new note")
            post(AddNoteBody(self.name, step, note, velocity, duration).json(), "sequence/add-note")

        return self

    def play(self):
        post(json.dumps(self.name), "sequence/play-one")
        log.info("play quedued...")

    def stop(self, now: bool = False):
        endpoint = "sequence/queue-stop"
        mesg = "stop queued..."

        if now:
            endpoint = "sequence/stop-these"
            mesg = "stopped..."

        post(json.dumps([self.name]), endpoint)
        log.info(mesg)

    def stop_now(self,):
        self.stop(True)


if __name__ == "__main__":
    import time

    set_log_level(logging.DEBUG)
    new_dev("VITAL")
    name = "test-sequence"
    post(json.dumps(name), "sequence/rm")
    seq = Sequence(name)
    # print(tempo())
    # print(f"this sequence: {seq.name}")
    # print("all_seqeunces : ", get("sequence/names"))
    # print("before sequence set up : ", get("sequence", sequence=seq.name))
    # seq.set_note(0, "C4").set_note(3, "c3")
    # scale = Scale("A#", "blues")
    # print(f"scale is {scale}")
    # print(f"scale is {seq.scale()}")
    seq.scale(Scale("d#", "dorian")).dev("vital").duration(NoteLen.En(1)).step(0, "2").step(3, "5").step(6, "1").step(9, "6")
    # print("after sequence set up : ", get("sequence", sequence=seq.name))
    # print(*(msg for msg in [step for step in seq.seq.steps]))
    be_seq = BackEndSequence.from_json(get("sequence", convert=False, sequence=seq.name))
    # print(*(msg for msg in [step for step in be_seq.steps]))
    time.sleep(1)
    seq.play()
    time.sleep(6)
    seq.stop()

