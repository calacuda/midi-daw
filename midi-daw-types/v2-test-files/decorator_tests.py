from midi_daw_types import MidiChannel
from midi_daw_types import v2

print("\n\n", "=" * 30, "\n\n", sep="")
song = v2.MidiDaw("MIDI-DEV", MidiChannel.Ch1)

def factory(dev, channel=None, loops=None, block=None):
    def wrapper(func):
        from functools import partial

        # return partial(v2.my_decorator_factory(func, dev, channel), block=block, loops=loops)
        return partial(v2.my_decorator_factory(func, dev, channel, block), loops=loops)

    return wrapper


# @song.test_wrapper
# @v2.my_decorator_factory("midi-dev")
@factory("midi-dev", loops=3,  block=False)
def f(api):
    print(f"playing f on {api.device}:{api.channel}")
    

@factory("diferent-dev", loops=3,  block=False)
def f_2(api):
    print(f"playing f_2 on {api.device}:{api.channel}")


@factory("a-third-dev", loops=3,  block=False)
def f_3(api):
    print(f"playing f_3 on {api.device}:{api.channel}")
 

if __name__ == "__main__":
    import time

    f(loops=10)
    f_2(loops=10)
    f_3(loops=10, block=True)
    print("called")
    time.sleep(2.5)
    print("exiting")
    
    # print(v2.my_decorator_factory("midi-dev"))
    # print(type(v2.my_decorator_factory("midi-dev")))
    # print(v2.my_decorator_factory("midi-dev")(f))
