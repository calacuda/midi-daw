import pygame
from pathlib import Path
from py_bevy import App, Schedule


_FRAME = 0
_FPS = 0.0
THIS_DIR = Path(__file__)
THIS_DIR = THIS_DIR.parent.absolute()
SCREEN_WIDTH = 1280
SCREEN_HEIGHT = 720
SCREEN_SIZE = (SCREEN_WIDTH, SCREEN_HEIGHT)


class States(Enum):
    STARTUP = 0
    # SELECT_COM_PORT = 1
    # SELECT_SCRIPT = 2
    # CONFIRM_SCRIPT = 3
    # MONITOR_SCRIPT = 4
    # SCRIPT_COMPLETE = 5
    EXIT = 1


app = App(States.STARTUP, DEBUG)


def run_app():
    pygame.init()
    pygame.joystick.init()
    should_loop = True

    while should_loop:
        # app.resources[ResourceID.PG_EVENTS].clear()

        for event in pygame.event.get():
            if event.type == pygame.QUIT:
                # print(event)
                should_loop = False
                continue
            elif event.type == pygame.JOYDEVICEADDED:
                joy = pygame.joystick.Joystick(event.device_index)
                # app.resources[ResourceID.CONTROLLER] = joy
            else:
                # app.resources[ResourceID.PG_EVENTS].append(event)
                pass

        # clear_screen(app)
        # controls_step(app)
        # app.step()
        # render(app)

    pygame.quit()
    quit()


if __name__ == "__main__":
    run_app()
