# from midi_daw import *
import time

from midi_daw_types import MidiChannel, NoteLen, v2

print("\n\n", "=" * 30, "\n\n", sep="")
song = v2.MidiDaw("Vital", MidiChannel.Ch1, virt=True)


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
    print(f"func-name = {f.__name__}")
    api.play("g4", NoteLen.Sn(1))
    print(f"playing f on {api.device}:{api.channel}")
    api.rest(NoteLen.Sn(1))


@play_on("diferent-dev", loops=3, block=False)
def f_2(api):
    print(f"playing f_2 on {api.device}:{api.channel}")
    time.sleep(0.15)


@play_on("a-third-dev", loops=3, block=False)
def f_3(api):
    print(f"func-name = {f.__name__}")
    api.play(f"c4", NoteLen.Sn(1))
    print(f"playing f_3 on {api.device}:{api.channel}")
    time.sleep(0.1)


if __name__ == "__main__":

    f(loops=16, block=True)
    f_2(loops=10)
    print("non-blockers called")
    # f_3(loops=10, block=True)
    print("blocking thread done")
    # time.sleep(2.5)
    print("exiting")

    # print(v2.my_decorator_factory("midi-dev"))
    # print(type(v2.my_decorator_factory("midi-dev")))
    # print(v2.my_decorator_factory("midi-dev")(f))
