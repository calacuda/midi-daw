import pygame
from pathlib import Path
from py_bevy import App, Schedule
from enum import Enum
from logging import DEBUG


# _FRAME = 0
# _FPS = 0.0
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
    PLAYING = 2


app = App(States.STARTUP, DEBUG)


class ResourceID(Enum):
    BG_COLOR = 0
    TEXT_COLOR = 1
    SCREEN = 2
    FONT = 3
    TO_RENDER = 4
    CLOCK = 5
    PG_EVENTS = 6
    HIGHLIGHT_COLOR = 7
    ROW_SIZE = 8
    HIGHLIGHT_SIZE = 9
    # CHAR_SIZE = 10
    CONTROLLER = 11
    KEY_SCAN = 12
    LAST_KEY_SCAN = 13
    FONT_2 = 14
    EXECUTER = 15
    SERIAL_PORTS = 16
    PORT_SELECT_BOX = 17
    SELECTED_PORT = 18
    SCRIPTS = 19
    SCRIPTS_I = 20
    SCRIPT_NAMES_BOX = 21
    SCRIPT_DOCS_BOX = 22
    EXEC = 23
    EXEC_THREAD = 24


class Buttons(Enum):
    UP = 0
    DOWN = 1
    LEFT = 2
    RIGHT = 3
    A = 4
    B = 5
    X = 6
    Y = 7
    START = 8
    SELECT = 9
    L_BUMP = 10
    R_BUMP = 11
    L_TRIG = 12
    R_TRIG = 13


def clear_screen(app):
    screen = app.resources.get(ResourceID.SCREEN)

    if screen:
        screen.fill(app.resources.get(ResourceID.BG_COLOR))


def init_screen(app):
    screen = pygame.display.set_mode(SCREEN_SIZE)
    app.resources[ResourceID.SCREEN] = screen


def init_font(app):
    pygame.font.init()
    font = pygame.font.Font(f'{THIS_DIR}/Anonymous Pro.ttf', 27)
    app.resources[ResourceID.FONT] = font

    font = pygame.font.Font(f'{THIS_DIR}/Anonymous Pro.ttf', 40)
    app.resources[ResourceID.FONT_2] = font


def init_clock(app):
    clock = pygame.time.Clock()
    app.resources[ResourceID.CLOCK] = clock


def init(app):
    # pygame.init()
    app.resources[ResourceID.BG_COLOR] = (0, 0, 0)
    app.resources[ResourceID.TEXT_COLOR] = (50, 255, 100)
    app.resources[ResourceID.HIGHLIGHT_COLOR] = (5, 50, 25)
    app.resources[ResourceID.TO_RENDER] = []
    app.resources[ResourceID.KEY_SCAN] = []
    app.resources[ResourceID.LAST_KEY_SCAN] = []

    init_screen(app)
    init_font(app)
    init_clock(app)

    clear_screen(app)


def render(app):
    if app.resources.get(ResourceID.TO_RENDER) and app.resources[ResourceID.SCREEN]:
        renders = app.resources.get(ResourceID.TO_RENDER)

        if renders:
            screen = app.resources[ResourceID.SCREEN]
            # print(renders)
            [screen.blit(*blit_args) for blit_args in renders]

        app.resources[ResourceID.TO_RENDER] = []

    pygame.display.update()


@app.register(States.STARTUP, Schedule.ENTER)
def init_setup(app: App):
    init(app)


@app.register(States.STARTUP, Schedule.UPDATE)
def switch(app: App):
    app.set_next_state(States.PLAYING)


@app.register(States.STARTUP, Schedule.EXIT)
def log_exit_setup(app: App):
    app.log.log(f"leaving state {app._state.state}")


def controls_step(app: App):
    for event in app.resources[ResourceID.PG_EVENTS]:
        if event.type == pygame.JOYHATMOTION and event.value == (0, 1):
            app.resources[ResourceID.KEY_SCAN].append(Buttons.UP)
        elif event.type == pygame.JOYHATMOTION and event.value == (0, -1):
            app.resources[ResourceID.KEY_SCAN].append(Buttons.DOWN)
        elif event.type == pygame.JOYHATMOTION and event.value == (-1, 0):
            app.resources[ResourceID.KEY_SCAN].append(Buttons.LEFT)
        elif event.type == pygame.JOYHATMOTION and event.value == (1, 0):
            app.resources[ResourceID.KEY_SCAN].append(Buttons.RIGHT)
        elif event.type == pygame.JOYBUTTONUP and event.button == 1:
            app.resources[ResourceID.KEY_SCAN].append(Buttons.A)
        elif event.type == pygame.JOYBUTTONUP and event.button == 0:
            app.resources[ResourceID.KEY_SCAN].append(Buttons.B)
        elif event.type == pygame.JOYBUTTONUP and event.button == 3:
            app.resources[ResourceID.KEY_SCAN].append(Buttons.X)
        elif event.type == pygame.JOYBUTTONUP and event.button == 4:
            app.resources[ResourceID.KEY_SCAN].append(Buttons.Y)
        elif event.type == pygame.JOYBUTTONUP and event.button == 7:
            app.resources[ResourceID.KEY_SCAN].append(Buttons.START)
        elif event.type == pygame.JOYBUTTONDOWN and event.button == 6:
            app.resources[ResourceID.KEY_SCAN].append(Buttons.SELECT)
        elif event.type == pygame.JOYBUTTONUP and event.button == 6:
            app.resources[ResourceID.KEY_SCAN] = [
                button for button in app.resources[ResourceID.KEY_SCAN] if button != Buttons.SELECT]
        elif event.type == pygame.JOYBUTTONUP and event.button == 4:
            app.resources[ResourceID.KEY_SCAN].append(Buttons.L_BUMP)
        elif event.type == pygame.JOYBUTTONUP and event.button == 4:
            app.resources[ResourceID.KEY_SCAN].append(Buttons.R_BUMP)
        elif event.type == pygame.JOYAXISMOTION and event.axis == 4 and event.value < 0.5:
            app.resources[ResourceID.KEY_SCAN].append(Buttons.L_TRIG)
        elif event.type == pygame.JOYAXISMOTION and event.axis == 5 and event.value < 0.5:
            app.resources[ResourceID.KEY_SCAN].append(Buttons.R_TRIG)
        elif event.type == pygame.KEYDOWN and event.key == pygame.K_UP:
            app.resources[ResourceID.KEY_SCAN].append(Buttons.UP)
        elif event.type == pygame.KEYDOWN and event.key == pygame.K_DOWN:
            app.resources[ResourceID.KEY_SCAN].append(Buttons.DOWN)
        elif event.type == pygame.KEYDOWN and event.key == pygame.K_LEFT:
            app.resources[ResourceID.KEY_SCAN].append(Buttons.LEFT)
        elif event.type == pygame.KEYDOWN and event.key == pygame.K_RIGHT:
            app.resources[ResourceID.KEY_SCAN].append(Buttons.RIGHT)
        elif event.type == pygame.KEYDOWN and event.key == pygame.K_x:
            app.resources[ResourceID.KEY_SCAN].append(Buttons.A)
        elif event.type == pygame.KEYDOWN and event.key == pygame.K_z:
            app.resources[ResourceID.KEY_SCAN].append(Buttons.B)


def render_stepper_col(seq_name, target, left, line_h, font, text_color, font_w):
    # for i in range(2):
    top_y = line_h
    text = font.render(seq_name[0:18], True, text_color)
    app.resources[ResourceID.TO_RENDER].append((text, (left, top_y)))
    top_y = line_h * 2
    text = font.render(target[0:18], True, text_color)
    app.resources[ResourceID.TO_RENDER].append((text, (left, top_y)))

    for i in range(2, 18):
        top = line_h * (i + 1)

        # TODO: write rust backend.
        # TODO: get step data from rust backend.

        left_offset = left
        text = font.render("---", True, text_color)
        app.resources[ResourceID.TO_RENDER].append((text, (left_offset, top)))

        left_offset += (font_w * 4)
        text = font.render("---", True, text_color)
        app.resources[ResourceID.TO_RENDER].append((text, (left_offset, top)))

        left_offset += (font_w * 4)
        text = font.render("----", True, text_color)
        app.resources[ResourceID.TO_RENDER].append((text, (left_offset, top)))

        left_offset += (font_w * 5)
        text = font.render("----", True, text_color)
        app.resources[ResourceID.TO_RENDER].append((text, (left_offset, top)))


@app.register(States.PLAYING, Schedule.UPDATE)
def draw_gui(app: App):
    _, px_height = SCREEN_SIZE
    line_h = px_height / 20
    font = app.resources[ResourceID.FONT]
    font_w, _ = font.size("M")
    line_n_col_w = font_w * 6
    reg_line_w = font_w * 19
    text_color = app.resources[ResourceID.TEXT_COLOR]

    for i in range(2, 18):
        top = line_h * (i + 1)
        text = font.render(f"{i - 1:>2}", True, text_color)
        app.resources[ResourceID.TO_RENDER].append(
            (text, (font_w * 2.25, top)))

    for i in range(3):
        left = line_n_col_w + reg_line_w * i

        render_stepper_col(
            f"sequence-{i}", f"target-{i}", left, line_h, font, text_color, font_w)


def run_app():
    pygame.init()
    pygame.joystick.init()
    # should_loop = True
    joy = None
    app.resources[ResourceID.PG_EVENTS] = []
    app.resources[ResourceID.CONTROLLER] = joy

    while app.should_loop():
        app.resources[ResourceID.PG_EVENTS].clear()

        for event in pygame.event.get():
            if event.type == pygame.QUIT:
                app.exit()
            elif event.type == pygame.JOYDEVICEADDED:
                joy = pygame.joystick.Joystick(event.device_index)
                app.resources[ResourceID.CONTROLLER] = joy
            else:
                app.resources[ResourceID.PG_EVENTS].append(event)
                pass

        clear_screen(app)
        controls_step(app)
        app.step()
        render(app)

    pygame.quit()


if __name__ == "__main__":
    run_app()
