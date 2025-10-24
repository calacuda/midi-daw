import pygame
from pathlib import Path
from py_bevy import App, Schedule
from enum import Enum
from logging import INFO
from midi_daw_back import MidiOut
from midi_daw_types import note_from_str
from copy import deepcopy


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


app = App(States.STARTUP, INFO)


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
    MIDI_OUTPUT = 25
    MAIN_CURSOR_POS = 26
    MOVE_TIMER = 27
    NOTE_TIMER = 27


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
    MENU = 14


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
    # TODO: consider making this a HashSet
    app.resources[ResourceID.KEY_SCAN] = []
    app.resources[ResourceID.LAST_KEY_SCAN] = []
    app.resources[ResourceID.MAIN_CURSOR_POS] = [0, 2]
    app.resources[ResourceID.MOVE_TIMER] = pygame.time.get_ticks()

    init_screen(app)
    init_font(app)
    init_clock(app)

    clear_screen(app)


def render(app):
    if app.resources.get(ResourceID.TO_RENDER) and app.resources[ResourceID.SCREEN]:
        renders = app.resources.get(ResourceID.TO_RENDER)

        def blit(screen, *args):
            try:
                screen.blit(*args)
            except Exception as e:
                print(*args)
                raise e

        if renders:
            screen = app.resources[ResourceID.SCREEN]
            # [screen.blit(*blit_args) for blit_args in renders]
            [blit(screen, *blit_args) for blit_args in renders]

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
        if event.type == pygame.JOYBUTTONDOWN:
            app.log.info(str(event))

        if event.type == pygame.JOYHATMOTION and event.value[1] == 1:
            app.resources[ResourceID.KEY_SCAN].append(Buttons.UP)
        elif event.type == pygame.JOYHATMOTION and event.value[1] == -1:
            app.resources[ResourceID.KEY_SCAN].append(Buttons.DOWN)
        elif event.type == pygame.JOYHATMOTION and event.value[0] == -1:
            app.resources[ResourceID.KEY_SCAN].append(Buttons.LEFT)
        elif event.type == pygame.JOYHATMOTION and event.value[0] == 1:
            app.resources[ResourceID.KEY_SCAN].append(Buttons.RIGHT)
        elif event.type == pygame.JOYBUTTONDOWN and event.button == 1:
            app.resources[ResourceID.KEY_SCAN].append(Buttons.B)
        elif event.type == pygame.JOYBUTTONDOWN and event.button == 0:
            app.resources[ResourceID.KEY_SCAN].append(Buttons.A)
        elif event.type == pygame.JOYBUTTONDOWN and event.button == 2:
            app.resources[ResourceID.KEY_SCAN].append(Buttons.X)
        elif event.type == pygame.JOYBUTTONDOWN and event.button == 3:
            app.resources[ResourceID.KEY_SCAN].append(Buttons.Y)
        elif event.type == pygame.JOYBUTTONDOWN and event.button == 9:
            app.resources[ResourceID.KEY_SCAN].append(Buttons.START)
        elif event.type == pygame.JOYBUTTONDOWN and event.button == 8:
            app.resources[ResourceID.KEY_SCAN].append(Buttons.SELECT)
        # elif event.type == pygame.JOYBUTTONUP and event.button == 6:
        #     app.resources[ResourceID.KEY_SCAN] = [
        #         button for button in app.resources[ResourceID.KEY_SCAN] if button != Buttons.SELECT]
        elif event.type == pygame.JOYBUTTONDOWN and event.button == 4:
            app.resources[ResourceID.KEY_SCAN].append(Buttons.L_BUMP)
        elif event.type == pygame.JOYBUTTONDOWN and event.button == 6:
            app.resources[ResourceID.KEY_SCAN].append(Buttons.L_TRIG)
        elif event.type == pygame.JOYBUTTONDOWN and event.button == 5:
            app.resources[ResourceID.KEY_SCAN].append(Buttons.R_BUMP)
        elif event.type == pygame.JOYBUTTONDOWN and event.button == 7:
            app.resources[ResourceID.KEY_SCAN].append(Buttons.R_TRIG)
        elif event.type == pygame.JOYBUTTONDOWN and event.button == 10:
            app.resources[ResourceID.KEY_SCAN].append(Buttons.MENU)
        elif event.type == pygame.JOYAXISMOTION and event.axis == 4 and event.value < 0.5:
            app.resources[ResourceID.KEY_SCAN].append(Buttons.L_TRIG)
        elif event.type == pygame.JOYAXISMOTION and event.axis == 5 and event.value < 0.5:
            app.resources[ResourceID.KEY_SCAN].append(Buttons.R_TRIG)
        elif event.type == pygame.JOYHATMOTION and event.value == (0, 0):
            app.resources[ResourceID.KEY_SCAN] = [button for button in app.resources[ResourceID.KEY_SCAN]
                                                  if button is not Buttons.UP and button is not Buttons.DOWN and button is not Buttons.LEFT and button is not Buttons.RIGHT]
        elif event.type == pygame.JOYHATMOTION and event.value[1] == 0:
            # app.resources[ResourceID.KEY_SCAN].append(Buttons.UP)
            # print("rm'ing up and down presses")
            app.resources[ResourceID.KEY_SCAN] = [button for button in app.resources[ResourceID.KEY_SCAN]
                                                  if button is not Buttons.UP and button is not Buttons.DOWN]
        # elif event.type == pygame.JOYHATMOTION and event.value[0] == 0:
        #     app.resources[ResourceID.KEY_SCAN].append(Buttons.DOWN)
        elif event.type == pygame.JOYHATMOTION and event.value[0] == 0:
            # app.resources[ResourceID.KEY_SCAN].append(Buttons.LEFT)
            # print("rm'ing left and right presses")
            app.resources[ResourceID.KEY_SCAN] = [button for button in app.resources[ResourceID.KEY_SCAN]
                                                  if button is not Buttons.LEFT and button is not Buttons.RIGHT]
        # elif event.type == pygame.JOYHATMOTION and event.value[0] == 0:
        #     app.resources[ResourceID.KEY_SCAN].append(Buttons.RIGHT)

        elif event.type == pygame.JOYBUTTONUP and event.button == 1:
            # app.resources[ResourceID.KEY_SCAN].append(Buttons.A)
            # print("rm'ing button A")
            app.resources[ResourceID.KEY_SCAN] = [
                button for button in app.resources[ResourceID.KEY_SCAN] if button is not Buttons.B]
        elif event.type == pygame.JOYBUTTONUP and event.button == 0:
            # app.resources[ResourceID.KEY_SCAN].append(Buttons.B)
            app.resources[ResourceID.KEY_SCAN] = [
                button for button in app.resources[ResourceID.KEY_SCAN] if button is not Buttons.A]
        elif event.type == pygame.JOYBUTTONUP and event.button == 2:
            # app.resources[ResourceID.KEY_SCAN].append(Buttons.X)
            app.resources[ResourceID.KEY_SCAN] = [
                button for button in app.resources[ResourceID.KEY_SCAN] if button is not Buttons.X]
        elif event.type == pygame.JOYBUTTONUP and event.button == 3:
            # app.resources[ResourceID.KEY_SCAN].append(Buttons.Y)
            app.resources[ResourceID.KEY_SCAN] = [
                button for button in app.resources[ResourceID.KEY_SCAN] if button is not Buttons.Y]
        elif event.type == pygame.JOYBUTTONUP and event.button == 9:
            # app.resources[ResourceID.KEY_SCAN].append(Buttons.START)
            app.resources[ResourceID.KEY_SCAN] = [
                button for button in app.resources[ResourceID.KEY_SCAN] if button is not Buttons.START]
        elif event.type == pygame.JOYBUTTONUP and event.button == 8:
            # app.resources[ResourceID.KEY_SCAN].append(Buttons.SELECT)
            app.resources[ResourceID.KEY_SCAN] = [
                button for button in app.resources[ResourceID.KEY_SCAN] if button is not Buttons.SELECT]
        # elif event.type == pygame.JOYBUTTONDOWN and event.button == 6:
        #     # app.resources[ResourceID.KEY_SCAN] = [
        #     #     button for button in app.resources[ResourceID.KEY_SCAN] if button != Buttons.SELECT]
        #
        #     app.resources[ResourceID.KEY_SCAN] = [button for button in app.resources[ResourceID.KEY_SCAN] if button is not Buttons.A]
        elif event.type == pygame.JOYBUTTONUP and event.button == 4:
            # app.resources[ResourceID.KEY_SCAN].append(Buttons.L_BUMP)
            app.resources[ResourceID.KEY_SCAN] = [
                button for button in app.resources[ResourceID.KEY_SCAN] if button is not Buttons.L_BUMP]
        elif event.type == pygame.JOYBUTTONUP and event.button == 6:
            # app.resources[ResourceID.KEY_SCAN].append(Buttons.L_BUMP)
            app.resources[ResourceID.KEY_SCAN] = [
                button for button in app.resources[ResourceID.KEY_SCAN] if button is not Buttons.L_TRIG]
        elif event.type == pygame.JOYBUTTONUP and event.button == 5:
            # app.resources[ResourceID.KEY_SCAN].append(Buttons.R_BUMP)
            app.resources[ResourceID.KEY_SCAN] = [
                button for button in app.resources[ResourceID.KEY_SCAN] if button is not Buttons.R_BUMP]
        elif event.type == pygame.JOYBUTTONUP and event.button == 7:
            # app.resources[ResourceID.KEY_SCAN].append(Buttons.R_BUMP)
            app.resources[ResourceID.KEY_SCAN] = [
                button for button in app.resources[ResourceID.KEY_SCAN] if button is not Buttons.R_TRIG]
        elif event.type == pygame.JOYBUTTONDOWN and event.button == 10:
            app.resources[ResourceID.KEY_SCAN] = [
                button for button in app.resources[ResourceID.KEY_SCAN] if button is not Buttons.MENU]
        elif event.type == pygame.JOYAXISMOTION and event.axis == 4 and event.value > 0.5:
            # app.resources[ResourceID.KEY_SCAN].append(Buttons.L_TRIG)
            app.resources[ResourceID.KEY_SCAN] = [
                button for button in app.resources[ResourceID.KEY_SCAN] if button is not Buttons.L_TRIG]
        elif event.type == pygame.JOYAXISMOTION and event.axis == 5 and event.value > 0.5:
            # app.resources[ResourceID.KEY_SCAN].append(Buttons.R_TRIG)
            app.resources[ResourceID.KEY_SCAN] = [
                button for button in app.resources[ResourceID.KEY_SCAN] if button is not Buttons.R_TRIG]


def render_stepper_col(app, screen, midi, seq_name, target, left, line_h, font, text_color, accent_color, font_w, col_number, cursor_pos):
    top_y = line_h
    # screen = app.resources[ResourceID.SCREEN]
    (cursor_x, cursor_y) = cursor_pos

    text = font.render(seq_name[0:18], True, text_color)

    if cursor_y == 0 and (cursor_x // 4) == col_number:
        rect = text.get_rect(topleft=(left - 2.5, top_y))
        pygame.draw.rect(screen, accent_color, rect.inflate(15, 15), 5)

    app.resources[ResourceID.TO_RENDER].append((text, (left, top_y)))
    top_y = line_h * 2
    text = font.render(target[0:18], True, text_color)

    if cursor_y == 1 and (cursor_x // 4) == col_number:
        rect = text.get_rect(topleft=(left - 2.5, top_y))
        pygame.draw.rect(screen, accent_color, rect.inflate(15, 15), 5)

    app.resources[ResourceID.TO_RENDER].append((text, (left, top_y)))

    def cell_selected(col, i):
        return cursor_y == i and cursor_x == (4 * col_number + col)

    seq_rows = midi.get_seq(seq_name)

    for i in range(2, 18):
        top = line_h * (i + 1)

        # get step data from rust backend.
        # row = midi.get_seq_row(seq_name, i - 2)
        row = seq_rows[i - 2]

        left_offset = left
        text = font.render(row[0], True, text_color)

        if cell_selected(0, i):
            rect = text.get_rect(topleft=(left_offset - 2.5, top))
            pygame.draw.rect(screen, accent_color, rect.inflate(15, 15), 5)

        app.resources[ResourceID.TO_RENDER].append((text, (left_offset, top)))

        left_offset += (font_w * 4)
        text = font.render(row[1], True, text_color)

        if cell_selected(1, i):
            rect = text.get_rect(topleft=(left_offset - 2.5, top))
            pygame.draw.rect(screen, accent_color, rect.inflate(15, 15), 5)

        app.resources[ResourceID.TO_RENDER].append((text, (left_offset, top)))

        left_offset += (font_w * 4)
        text = font.render(row[2], True, text_color)

        if cell_selected(2, i):
            rect = text.get_rect(topleft=(left_offset - 2.5, top))
            pygame.draw.rect(screen, accent_color, rect.inflate(15, 15), 5)

        app.resources[ResourceID.TO_RENDER].append((text, (left_offset, top)))

        left_offset += (font_w * 5)
        text = font.render(row[3], True, text_color)

        if cell_selected(3, i):
            rect = text.get_rect(topleft=(left_offset - 2.5, top))
            pygame.draw.rect(screen, accent_color, rect.inflate(15, 15), 5)

        app.resources[ResourceID.TO_RENDER].append((text, (left_offset, top)))


@app.register(States.PLAYING, Schedule.UPDATE)
def draw_gui(app: App):
    px_width, px_height = SCREEN_SIZE
    line_h = px_height / 20
    font = app.resources[ResourceID.FONT]
    font_w, _ = font.size("M")
    line_n_col_w = font_w * 6
    reg_line_w = font_w * 19
    text_color = app.resources[ResourceID.TEXT_COLOR]
    accent_color = app.resources[ResourceID.HIGHLIGHT_COLOR]
    midi = app.resources[ResourceID.MIDI_OUTPUT]
    screen = app.resources[ResourceID.SCREEN]
    cursor_pos = app.resources[ResourceID.MAIN_CURSOR_POS]
    sequence_names = midi.get_seq_names()[:3]

    for i in range(2, 18):
        top = line_h * (i + 1)
        line_num = i - 1
        line_index = i - 2
        current_step = midi.get_step_n() == line_index

        text = font.render(f"{line_num:>2}", True,
                           text_color if not current_step else accent_color)

        app.resources[ResourceID.TO_RENDER].append(
            (text, (font_w * 2.25, top)))

    for i in range(3):
        left = line_n_col_w + reg_line_w * i
        (name, dev) = sequence_names[i]

        render_stepper_col(app, screen,
                           midi, name, dev, left, line_h, font, text_color, accent_color, font_w, i, cursor_pos)

    left = line_n_col_w + reg_line_w * 3 + font_w * 3
    top = line_h * 2
    bottom = px_height - top
    rect = pygame.Rect(left - 2.5, top, px_width - left, bottom - top)
    pygame.draw.rect(screen, accent_color, rect.inflate(15, 15), 5)


def move_timer_done(last_time):
    curent_time = pygame.time.get_ticks()
    delta = curent_time - last_time

    return delta > 100


@app.register(States.PLAYING, Schedule.UPDATE)
def move_main_cursor(app: App):
    key_scan = app.resources[ResourceID.KEY_SCAN]

    if Buttons.A in key_scan:
        return

    move_timer = app.resources[ResourceID.MOVE_TIMER]

    if move_timer_done(move_timer):
        app.resources[ResourceID.MOVE_TIMER] = pygame.time.get_ticks()

        if Buttons.RIGHT in key_scan:
            app.resources[ResourceID.MAIN_CURSOR_POS][0] += 1
            app.resources[ResourceID.MAIN_CURSOR_POS][0] %= 4 * 3
        if Buttons.LEFT in key_scan:
            app.resources[ResourceID.MAIN_CURSOR_POS][0] -= 1
            app.resources[ResourceID.MAIN_CURSOR_POS][0] %= 4 * 3
        if Buttons.UP in key_scan:
            app.resources[ResourceID.MAIN_CURSOR_POS][1] -= 1
            app.resources[ResourceID.MAIN_CURSOR_POS][1] %= 18
        if Buttons.DOWN in key_scan:
            app.resources[ResourceID.MAIN_CURSOR_POS][1] += 1
            app.resources[ResourceID.MAIN_CURSOR_POS][1] %= 18
    elif Buttons.LEFT not in key_scan and Buttons.RIGHT not in key_scan and Buttons.UP not in key_scan and Buttons.DOWN not in key_scan:
        app.resources[ResourceID.MOVE_TIMER] = 0


@app.register(States.PLAYING, Schedule.UPDATE)
def adjust_note(app: App):
    key_scan = app.resources[ResourceID.KEY_SCAN]
    (cursor_x, cursor_y) = app.resources[ResourceID.MAIN_CURSOR_POS]

    if Buttons.A not in key_scan or cursor_x % 4 != 0 or cursor_y < 2:
        return

    note_timer = app.resources[ResourceID.NOTE_TIMER]

    if move_timer_done(note_timer):
        app.resources[ResourceID.NOTE_TIMER] = pygame.time.get_ticks()
        midi = app.resources[ResourceID.MIDI_OUTPUT]
        sequence_names = midi.get_seq_names()[:3]
        seq = sequence_names[cursor_x // 4][0]
        note, _, _, _ = midi.get_seq_row(seq, cursor_y - 2)
        note = note.replace("-", "")
        # print(f"note_from_str({note}) = {note_from_str(note)}")
        note = note_from_str(note)

        if note is not None:
            note -= 24

            if Buttons.RIGHT in key_scan:
                new_note = (note + 12) % 104 + 24
                # print(note, "->", new_note)
                midi.set_note(seq, cursor_y - 2, new_note, None)
            elif Buttons.LEFT in key_scan:
                new_note = (note - 12) % 104 + 24
                # print(note, "->", new_note)
                midi.set_note(seq, cursor_y - 2, new_note, None)
            elif Buttons.UP in key_scan:
                new_note = (note + 1) % 104 + 24
                # print(note, "->", new_note)
                midi.set_note(seq, cursor_y - 2, new_note, None)
            elif Buttons.DOWN in key_scan:
                new_note = (note - 1) % 104 + 24
                # print(note, "->", new_note)
                midi.set_note(seq, cursor_y - 2, new_note, None)
            elif Buttons.B in key_scan:
                midi.set_note(seq, cursor_y - 2, None, None)
        else:
            midi.set_note(seq, cursor_y - 2, 24, None)


@app.register(States.PLAYING, Schedule.UPDATE)
def change_midi_target(app: App):
    key_scan = app.resources[ResourceID.KEY_SCAN]
    (cursor_x, cursor_y) = app.resources[ResourceID.MAIN_CURSOR_POS]

    if Buttons.A not in key_scan or cursor_y != 1:
        return

    # timer = app.resources[ResourceID.NOTE_TIMER]

    if move_timer_done(app.resources[ResourceID.NOTE_TIMER]):
        app.resources[ResourceID.NOTE_TIMER] = pygame.time.get_ticks()
        midi = app.resources[ResourceID.MIDI_OUTPUT]
        devices = midi.list_devs()
        # print(f"devices: {devices}")
        sequence_names = midi.get_seq_names()[:3]
        seq_name, old_dev = sequence_names[cursor_x // 4]
        i = 0

        if old_dev in devices:
            i = devices.index(old_dev)

            if Buttons.RIGHT in key_scan:
                i = (i + 1) % len(devices)
            elif Buttons.LEFT in key_scan:
                i = (i - 1) % len(devices)

        dev = devices[i % len(devices)]

        if dev is not None:
            midi.change_sequence_dev(seq_name, dev)


@app.register(States.PLAYING, Schedule.UPDATE)
def set_playing(app: App):
    key_scan = app.resources[ResourceID.KEY_SCAN]
    last_key_scan = app.resources[ResourceID.LAST_KEY_SCAN]

    if Buttons.START in key_scan and Buttons.START not in last_key_scan:
        cursor_x, _ = app.resources[ResourceID.MAIN_CURSOR_POS]
        midi = app.resources[ResourceID.MIDI_OUTPUT]
        sequence_names = midi.get_seq_names()[:3]
        seq_name, _ = sequence_names[cursor_x // 4]
        print(seq_name)
        midi.play_seq(seq_name)


# TODO: add sidebar menu
# TODO: add adding sequences
# TODO: add chains (a sequence of sequences)
# TODO: add tempo changing
# TODO: add cmds


@app.register(States.STARTUP, Schedule.UPDATE)
@app.register(States.PLAYING, Schedule.UPDATE)
def exit_game(app: App):
    key_scan = app.resources[ResourceID.KEY_SCAN]

    if Buttons.SELECT in key_scan and Buttons.MENU in key_scan and len(key_scan) == 2:
        app.exit()


def run_app():
    pygame.init()
    pygame.joystick.init()
    # should_loop = True
    joy = None
    app.resources[ResourceID.PG_EVENTS] = []
    app.resources[ResourceID.CONTROLLER] = joy
    midi = MidiOut()
    app.resources[ResourceID.MIDI_OUTPUT] = midi

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
        app.resources[ResourceID.LAST_KEY_SCAN] = deepcopy(
            app.resources[ResourceID.KEY_SCAN])
        # app.resources[ResourceID.KEY_SCAN].clear()
        app.resources[ResourceID.CLOCK].tick()

    pygame.quit()


if __name__ == "__main__":
    run_app()
