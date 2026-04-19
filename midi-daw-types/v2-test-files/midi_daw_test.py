from midi_daw_types import MidiChannel, NoteLen, v2
from midi_daw_types.v2 import lfo, main, wait_for_bar
from functools import partial
from os import environ, path
from loguru import logger as log
from atomicx import AtomicBool
from enum import Enum, auto
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

# maps func-name to the atomic bool that controls stopping
THREADS = {}
# a dict storing assiociating instrument with a way to stop it
INSTRUMENTS = {}


def play_on(dev, channel=None, is_virt=True, loops=None, block=None, start_on_bar=True):
    from functools import wraps
    from midi_daw_types.v2 import Api, find_dev, list_devs, wait_for_bar
    import threading
    import inspect
    import importlib

    def wrapper(func):
        full_dev_name = dev if (dev in list_devs()) or is_virt else find_dev(dev)
        api = Api(full_dev_name, channel if channel is not None else MidiChannel.Ch1, is_virt)
        loop_n = loops
        blocking = block
        f_name = func.__name__
        thread = None
        should_stop = AtomicBool()
        should_stop.store(False)
        
        @wraps(func)
        def run(*args, **kwargs):
            nonlocal loop_n
            nonlocal blocking
            nonlocal thread
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

                    if should_stop.load():
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

                    if should_stop.load():
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
                global THREADS
                nonlocal should_stop

                maybe_thread = THREADS.get(f_name)

                if maybe_thread is None:
                    should_stop.store(False)
                    run(*args, **kwargs)
                    THREADS[f_name] = should_stop
                if maybe_thread:
                    maybe_thread.store(False)
                    run(*args, **kwargs)

            def stop(self):
                nonlocal should_stop

                should_stop.store(True)

        # return f
        return Func()

    return wrapper


def stop_fn(f_name):
    global THREADS

    should_stop = THREADS.get(f_name)

    if should_stop is not None:
        log.info(f"queued stop for fucntion: {f_name}.")
        should_stop.store(True)
    else:
        log.warning(f"unknown function named, {f_name}. was it already stopped?")
        log.info(f"known functions: {THREADS.keys()}.")


def connect(source_name, sink_name):
    """connects a jack source to a jack sink"""
    pass


class JackPortType(Enum):
    MIDI = auto()
    AUDIO = auto()

    def __str__(self):
        if self is JackPortType.MIDI:
            return "MIDI"
        elif self is JackPortType.AUDIO:
            return "AUDIO"
        else:
            return "JackPortType failed to convert to str"

    def __repr__(self):
        return self.__str__()

if __name__ == "__main__":
    print(f"should say \"MIDI\" => \"{JackPortType.MIDI}\"")
    print([("FAKE-PORT-NAME", JackPortType.MIDI)])
    print(JackPortType.MIDI)
    print(f"should say \"AUDIO\" => \"{JackPortType.AUDIO}\"")
    print(JackPortType.AUDIO)


class Plugin:
    """
    repressents a single instance of a running plugin hosted on the sync thread
    """
    def __init__(self, plugin: str):
        """the plugin arguemnt can be a file path or name"""
        self.plugin = plugin
        # TODO: send request to start plugin
        # TODO: parse this from mk-plugin responce
        self.jack_name = ""
        # a list of (port_name, is_midi) representing inputs
        # TODO: parse from mk-plugin responce
        self.input_ports: list[(str, JackPortType)] = []
        # a list of (port_name, is_midi) representing outputs
        # TODO: parse from mk-plugin responce
        self.output_ports: list[(str, JackPortType)] = []

    @property
    def name(self):
        """gets the name of this plugins jacks client"""
        return self.jack_name

    @name.setter
    def name(self, new_name):
        """sets the name of the jacks client for this plugin"""
        # TODO: send signal to backend to change clients name

        self.jack_name = new_name

    def get_params(self):
        """returns a list of parameters this plugin has"""
        # TODO: get list of params

        return []

    def param(self, param: str, value=None):
        """returns the value of the parameter, or sets the value if provided"""
        # TODO: get/set param
        return None

    # def connect(self, src_name: str, sink_name: str):
    def connect(self, my_port_name: str, other_port_name: str):
        """
        connects the jacks src to the sink (this method assumes that ether the
        src or sink is an output or input of this plugin
        """
        in_names = [port[0] for port in self.input_ports]
        out_names = [port[0] for port in self.output_ports]

        if my_port_name in in_names:
            connect(other_port_name, f"{self.name}:{my_port_name}")
        elif my_port_name in out_names:
            connect(f"{self.name}:{my_port_name}", other_port_name)
        else:
            log.error(f"\"{my_port_name}\" is not a known input or output port of this plugin.")
            log.info(f"known inputs: {in_names}")
            log.info(f"known outputs: {out_names}")


