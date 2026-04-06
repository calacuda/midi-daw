# from midi_daw import *
import time

from midi_daw import MidiChannel, NoteLen, v2
from midi_daw.v2 import lfo, main
from midi_daw.v2.note_lens import *

print("\n\n", "=" * 30, "\n\n", sep="")
song = v2.MidiDaw("Vital", MidiChannel.Ch1, virt=True)
automa = v2.MidiDaw("Vital", MidiChannel.Ch1, virt=True)


def play_on(dev, channel=None, loops=None, block=None):
    from functools import partial, wraps

    def wrapper(func):
        # return partial(v2.my_decorator_factory(func, dev, channel), block=block, loops=loops)
        f = partial(v2.play_on(func, dev, channel, block), loops=loops)
        f = wraps(func)(f)

        return f

    return wrapper


# dev = v2.mk_dev("Vital")
# time.sleep(1)
devs = v2.list_devs()
print(devs)


# @song.test_wrapper
# @v2.my_decorator_factory("midi-dev")
# @factory("midi-dev", loops=3,  block=False)
@song.register
def f(api):
    # # print(f"func-name = {f.__name__}")
    # api.play("g4", NoteLen.Sn(7))
    # # print(f"py: f playing  playing g4 on {api.device}:{api.channel} again", flush=True)
    # # api.play("g4", NoteLen.Wn(1))
    # # time.sleep(1.0)
    # # print(f"py: playing g4 on {api.device}:{api.channel}")
    # api.rest(NoteLen.Sn(1))
    api.play("c4", sn(1))
    api.rest(sn(12 // 3 - 1))


@automa.register
@lfo.sin
def pitch_worble(api, lfo_sample):
    bend_amt = lfo_sample * 0.75
    # print(f"py: bending by, {bend_amt}", flush=True)
    api.pitch_bend(bend_amt)


# print(type(pitch_worble))


# @play_on("diferent-dev", loops=3, block=False)
@song.register
def f_2(api):
    # print(f"playing f_2 on {api.device}:{api.channel}")
    # time.sleep(0.15)
    api.play("b4", sn(1))
    api.rest(sn(12 // 2 - 1))


# @play_on("a-third-dev", loops=3, block=False)
@song.register
def f_3(api):
    # n = 2
    # # print(f"func-name = {f_3.__name__}")
    # api.play(f"c4", NoteLen.Sn(n))
    # # print(f"py: playing c_4 on {api.device}:{api.channel}")
    # api.rest(NoteLen.Sn(16 - n))
    # # time.sleep(0.1)
    api.play("g4", sn(1))
    api.rest(sn(12 // 2 - 1))


# run_main_if()
# if __name__ == "__main__":
#     loops = 128
#     try:
#         print(run_main_if())
#     except:
#         print("ERROR")


@main
def main():
    loops = 128
    pitch_jh = pitch_worble(freq=7.5, block=False)
    f_jh = f(loops=loops, block=False)
    print("non-blockers called")
    f_2_jh = f_2(loops=int(loops * (2 / 3)), block=False)
    f_3_jh = f_3(loops=int(loops * (2 / 3)), block=True)
    song.stop()
    print("blocking thread done")
    print("exiting")
    exit(0)

    # print(v2.my_decorator_factory("midi-dev"))
    # print(type(v2.my_decorator_factory("midi-dev")))
    # print(v2.my_decorator_factory("midi-dev")(f))
