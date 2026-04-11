from midi_daw import MidiChannel, NoteLen, v2
from midi_daw.v2 import lfo, main
from midi_daw.v2.note_lens import *

# lead = v2.MidiDaw("vital-lead", MidiChannel.Ch1, virt=True)
bass = v2.MidiDaw("Vital - bass:events-in", MidiChannel.Ch1, virt=False)
# API = v2.Api("vital-bass", MidiChannel.Ch1, True)
devs = v2.list_devs()
print(devs)

def play_on(dev, channel=None, is_virt=True, loops=None, block=None):
    from functools import wraps
    from midi_daw.v2 import Api, find_dev
    from reloadr import autoreload
    import threading

    threads = []

    def wrapper(func):
        api = Api(find_dev(dev), channel if channel is not None else MidiChannel.Ch1, is_virt)
        loop_n = loops
        blocking = block
        func = autoreload(func)

        @wraps(func)
        def f(*args, **kwargs):
            nonlocal loop_n
            nonlocal blocking
            nonlocal threads

            loop = kwargs.pop("loops", loop_n)
            loop = loop if loop is not None else loop_n if loop_n is not None else 1
            block = kwargs.pop("block", blocking)
            block = block if block is not None else blocking if blocking is not None else True

            def forever():
                while True:
                    func(api, *args, **kwargs)

            def for_not_ever():
                for i in range(loop):
                    # print(f"{i} -> {api.device}")
                    func(api, **kwargs)

            if block and loop > 0:
                for i in range(loop):
                    func(api, **kwargs)
            elif block and loop <= 0:
                while True:
                    func(api, **kwargs)
            elif not block and loop > 0:
                thread = threading.Thread(target=for_not_ever)
                thread.start()
                threads.append(thread)
            elif not block and loop <= 0:
                thread = threading.Thread(target=forever)
                thread.start()
                threads.append(thread)
            else:
                print("unreachable")

        return f

    return wrapper


@play_on("Vital - bass", is_virt=False)
def bass(m):
    m.seq("c4 <e4*2 b4> <c4*2 g4> ~ g4", sn()) 


@play_on("909", MidiChannel.Ch10, is_virt=False)
def kick(m):
    m.seq("c2 [c2 e2] c2 [c2 e2]", qn())


@play_on("909", MidiChannel.Ch10, is_virt=False)
def hh(m):
    m.seq("f#2 f#2 a#2 ~ ~ f#2 f#2 a#2 f#2 f#2 a#2 ~ ~ f#2 f#2 a#2", sn())


loops = 2

bass(loops=loops, block=False)
kick(loops=loops, block=False)
hh(loops=loops, block=True)

