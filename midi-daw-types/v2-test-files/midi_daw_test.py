from midi_daw_types import MidiChannel, NoteLen, v2
from midi_daw_types.v2 import lfo, main, wait_for_bar
from functools import partial
from sys import stderr


print = partial(print, file=stderr)


def play_on(dev, channel=None, is_virt=True, loops=None, block=None, start_on_bar=True):
    from functools import wraps
    from midi_daw_types.v2 import Api, find_dev, list_devs, wait_for_bar
    from reloading import reloading
    import threading
    import inspect
    import importlib

    def wrapper(func):
        full_dev_name = dev if (dev in list_devs()) or is_virt else find_dev(dev)
        api = Api(full_dev_name, channel if channel is not None else MidiChannel.Ch1, is_virt)
        loop_n = loops
        blocking = block
        f_name = func.__name__
        threads = None
        should_stop = False
        
        @wraps(func)
        def run(*args, **kwargs):
            nonlocal loop_n
            nonlocal blocking
            nonlocal threads
            nonlocal start_on_bar
            nonlocal should_stop

            loop = kwargs.pop("loops", loop_n)
            # print(f"{loop} > {loop_n} > {1}")
            loop = loop if loop is not None else loop_n if loop_n is not None else 1
            block = kwargs.pop("block", blocking)
            block = block if block is not None else blocking if blocking is not None else False 

            def wait_for_start():
                if start_on_bar:
                    wait_for_bar()

            @wraps(func)
            def forever():
                wait_for_start()
                print(f"about to start loop (will loop, indefinitely) for function: {f_name}", flush=True)
                i = 0

                for _ in reloading(forever=True):
                    func(api, **kwargs)
                    i += 1

                    if should_stop:
                        break
                    
                print(f"function: {f_name}, looped {i} times", flush=True)

            @wraps(func)
            def for_not_ever():
                wait_for_start()
                print(f"about to start loop (will loop, {loop}, times) for function: {f_name}", flush=True)
                i = 0
                
                for i in range(loop):
                    func(api, **kwargs) 
                    i += 1

                    if should_stop:
                        break

                print(f"function: {f_name}, looped {i} times", flush=True)

            if block and loop > 0:
                if start_on_bar:
                    api.wait_for_bar()

                for_not_ever()
            elif block and loop <= 0:
                if start_on_bar:
                    api.wait_for_bar()

                forever()
            elif not block and loop > 0:
                thread = threading.Thread(target=for_not_ever)
            elif not block and loop <= 0:
                thread = threading.Thread(target=forever)
            else:
                print("unreachable")

            if not block and thread is not None and not thread.is_alive():
                thread.start()
                print(f"start queued for function: {f_name}")

        # @wraps(func)
        class Func:
            def __init__(self):
                pass

            @wraps(func)
            def __call__(self, *args, **kwargs):
                nonlocal should_stop

                should_stop = False
                run(*args, **kwargs)

            def stop(self):
                nonlocal should_stop

                should_stop = True

        # return f
        return Func()

    return wrapper

