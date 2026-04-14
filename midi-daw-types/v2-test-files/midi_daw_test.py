from midi_daw_types import MidiChannel, NoteLen, v2
from midi_daw_types.v2 import lfo, main, wait_for_bar
from functools import partial
from os import environ, path
from loguru import logger as log
import builtins
import inspect
import logging 
import sys

location = environ.get("JURIGGGED_CODE_REDIRECT", None)
_log_fmt = "[<level>{level}</level>]: {message}"

if location is not None and path.exists(location):
    f = open(location, "a")
    # print = partial(builtins.print, file=f)
    log.remove()
    log.opt(colors=True).add(f, colorize=True, format=_log_fmt)
else:
    log.opt(colors=True).add(sys.stderr, colorize=True, format=_log_fmt)


class InterceptHandler(logging.Handler):
    def emit(self, record: logging.LogRecord) -> None:
        # Get corresponding Loguru level if it exists.
        try:
            level: str | int = log.level(record.levelname).name
        except ValueError:
            level = record.levelno

        # Find caller from where originated the logged message.
        frame, depth = inspect.currentframe(), 0
        while frame:
            filename = frame.f_code.co_filename
            is_logging = filename == logging.__file__
            is_frozen = "importlib" in filename and "_bootstrap" in filename
            if depth > 0 and not (is_logging or is_frozen):
                break
            frame = frame.f_back
            depth += 1

        log.opt(depth=depth, exception=record.exc_info).log(level, record.getMessage())

logging.basicConfig(handlers=[InterceptHandler()], level=0, force=True)
logging.getLogger('asyncio').setLevel(logging.WARNING)
watchdog_logger = logging.getLogger('watchdog')
watchdog_logger.addHandler(logging.NullHandler())
watchdog_logger.propagate = False
logging.getLogger('watchdog').setLevel(logging.WARNING)
logging.getLogger('watchdog').disabled = True
logging.getLogger('watchdog.observers.inotify_buffer').disabled = True


def play_on(dev, channel=None, is_virt=True, loops=None, block=None, start_on_bar=True):
    from functools import wraps
    from midi_daw_types.v2 import Api, find_dev, list_devs, wait_for_bar
    # from reloading import reloading
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
                import builtins
                
                # if location is not None and path.exists(location):
                #     with open(location, "a") as f:
                        # print = partial(builtins.print, file=f)

                wait_for_start()
                log.debug(f"about to start loop (will loop, indefinitely) for function: {f_name}")
                i = 0

                while True:
                    func(api, **kwargs)
                    i += 1

                    if should_stop:
                        break
                    
                log.debug(f"function: {f_name}, looped {i} times")

            @wraps(func)
            def for_not_ever():
                import builtins
                
                # if location is not None and path.exists(location):
                #     with open(location, "a") as f:
                #         # with redirect_stdout(f), redirect_stderr(f):
                #         # with redirect_stderr(f):
                #         print = partial(builtins.print, file=f)
                        
                # print(f"location to send std-out: {location}, {path.exists(location)}")
                wait_for_start()
                log.debug(f"about to start loop (will loop, {loop}, times) for function: {f_name}")
                i = 0
                
                for i in range(loop):
                    func(api, **kwargs) 
                    i += 1

                    if should_stop:
                        break

                log.debug(f"function: {f_name}, looped {i} times")

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
                log.fatal("unreachable")

            if not block and thread is not None and not thread.is_alive():
                thread.start()
                log.debug(f"start queued for function: {f_name}")

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

