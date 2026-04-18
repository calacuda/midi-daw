use std::{
    ops::Deref,
    panic,
    sync::{Arc, RwLock},
    time::Duration,
};

use ansi_to_tui::IntoText;
use color_eyre::Result;
use lazy_static::lazy_static;
use midi_daw_types::{
    tempo_from_bpm,
    v2::{BPQ, DEFAULT_BPM, SYNC_DEV_NAME, SYNC_DEV_PORT_NAME, TEMPO_SET_PORT},
};
use midi_msg::{Meta, MidiMsg, SystemRealTimeMsg};
use ratatui::{
    DefaultTerminal,
    crossterm::{
        self,
        event::{self, KeyCode, KeyEventKind},
    },
    prelude::*,
    widgets::*,
};
use tracing::*;
use tracing_subscriber::{EnvFilter, FmtSubscriber};
// use tracing_subscriber_multi::*;

lazy_static! {
    static ref LOGS: RwLock<Vec<String>> = RwLock::new(Vec::new());
    static ref COUNTER: Arc<RwLock<u32>> = Arc::new(RwLock::new(0));
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
struct App {
    /// Current value of the input box
    input: String,
    /// Position of cursor in the editor area.
    character_index: usize,
    /// Current input mode
    input_mode: InputMode,
    exit_trigger: bool,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
enum InputMode {
    #[default]
    Normal,
    Editing,
}

struct LogAppender;

impl std::io::Write for LogAppender {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let msg =
            String::from_utf8(buf.to_vec()).map_err(|e| std::io::Error::other(format!("{e}")))?;
        _ = LOGS.write().map(|mut logs| logs.push(msg.clone()));

        Ok(msg.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl App {
    pub const fn new() -> Self {
        Self {
            exit_trigger: false,
            input: String::new(),
            character_index: 0,
            input_mode: InputMode::Editing,
        }
    }

    fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.character_index.saturating_sub(1);
        self.character_index = self.clamp_cursor(cursor_moved_left);
    }

    fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.character_index.saturating_add(1);
        self.character_index = self.clamp_cursor(cursor_moved_right);
    }

    fn enter_char(&mut self, new_char: char) {
        let index = self.byte_index();
        self.input.insert(index, new_char);
        self.move_cursor_right();
    }

    /// Returns the byte index based on the character position.
    ///
    /// Since each character in a string can contain multiple bytes, it's necessary to calculate
    /// the byte index based on the index of the character.
    fn byte_index(&self) -> usize {
        self.input
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.character_index)
            .unwrap_or(self.input.len())
    }

    fn delete_char(&mut self) {
        let is_not_cursor_leftmost = self.character_index != 0;
        if is_not_cursor_leftmost {
            // Method "remove" is not used on the saved text for deleting the selected char.
            // Reason: Using remove on String works on bytes instead of the chars.
            // Using remove would require special care because of char boundaries.

            let current_index = self.character_index;
            let from_left_to_current_index = current_index - 1;

            // Getting all characters before the selected character.
            let before_char_to_delete = self.input.chars().take(from_left_to_current_index);
            // Getting all characters after selected character.
            let after_char_to_delete = self.input.chars().skip(current_index);

            // Put all characters together except the selected one.
            // By leaving the selected one out, it is forgotten and therefore deleted.
            self.input = before_char_to_delete.chain(after_char_to_delete).collect();
            self.move_cursor_left();
        }
    }

    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.input.chars().count())
    }

    const fn reset_cursor(&mut self) {
        self.character_index = 0;
    }

    fn submit_message(&mut self) {
        // self.messages.push(self.input.clone());
        let cmd = self.input.clone();
        _ = LOGS
            .write()
            .map(|mut logs| logs.push(format!("running cmd: \"{cmd}\"")));
        if cmd.eq_ignore_ascii_case("q")
            || cmd.eq_ignore_ascii_case("quit")
            || cmd.eq_ignore_ascii_case("exit")
        {
            self.exit_trigger = true;
        } else if cmd.eq_ignore_ascii_case("bar") || cmd.eq_ignore_ascii_case("step") {
            _ = COUNTER.write().map(|mut counter| *counter = 0);
        } else {
            error!("wrong or unknown command: \"{cmd}\"");
        }

        self.input.clear();
        self.reset_cursor();
    }

    fn run(mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        while !self.exit_trigger {
            terminal.draw(|frame| self.render(frame))?;

            if event::poll(Duration::from_millis(16))? {
                if let Some(key) = event::read()?.as_key_press_event() {
                    match self.input_mode {
                        InputMode::Normal => match key.code {
                            KeyCode::Char('e') => {
                                self.input_mode = InputMode::Editing;
                            }
                            KeyCode::Char('q') => {
                                return Ok(());
                            }
                            _ => {}
                        },
                        InputMode::Editing if key.kind == KeyEventKind::Press => match key.code {
                            KeyCode::Enter => self.submit_message(),
                            KeyCode::Char(to_insert) => self.enter_char(to_insert),
                            KeyCode::Backspace => self.delete_char(),
                            KeyCode::Left => self.move_cursor_left(),
                            KeyCode::Right => self.move_cursor_right(),
                            KeyCode::Esc => self.input_mode = InputMode::Normal,
                            _ => {}
                        },
                        InputMode::Editing => {}
                    }
                }
            }
        }

        Ok(())
    }

    fn render(&self, frame: &mut Frame) {
        let layout = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Min(1),
        ]);
        let [help_area, input_area, messages_area] = frame.area().layout(&layout);

        let (msg, style) = match self.input_mode {
            InputMode::Normal => (
                vec![
                    "Press ".into(),
                    "q".bold(),
                    " to exit, ".into(),
                    "e".bold(),
                    " to start editing.".bold(),
                ],
                Style::default().add_modifier(Modifier::RAPID_BLINK),
            ),
            InputMode::Editing => (
                vec![
                    "Press ".into(),
                    "Esc".bold(),
                    " to stop editing, ".into(),
                    "Enter".bold(),
                    " to record the message".into(),
                ],
                Style::default(),
            ),
        };
        let text = Text::from(Line::from(msg)).patch_style(style);
        let help_message = Paragraph::new(text);
        frame.render_widget(help_message, help_area);

        let input = Paragraph::new(self.input.as_str())
            .style(match self.input_mode {
                InputMode::Normal => Style::default(),
                InputMode::Editing => Style::default().fg(Color::Yellow),
            })
            .block(Block::bordered().title("Input"));
        frame.render_widget(input, input_area);
        match self.input_mode {
            // Hide the cursor. `Frame` does this by default, so we don't need to do anything here
            InputMode::Normal => {}

            // Make the cursor visible and ask ratatui to put it at the specified coordinates after
            // rendering
            #[expect(clippy::cast_possible_truncation)]
            InputMode::Editing => frame.set_cursor_position(Position::new(
                // Draw the cursor at the current position in the input field.
                // This position can be controlled via the left and right arrow key
                input_area.x + self.character_index as u16 + 1,
                // Move one line down, from the border to the input line
                input_area.y + 1,
            )),
        }

        // let mut messages: Vec<ListItem> = LOGS
        // let mut messages: Vec<Text> = LOGS
        let mut messages: Vec<String> = LOGS
            .read()
            .map(|logs| {
                logs.clone()
                //         // .enumerate()
                //         // .map(|(i, m)| {
                //         .map(|m| {
                //             // let content = Line::from(
                //             //     //     ratatui::prelude::Span::raw(format!(
                //             //     //     "{i}: {}",
                //             //     m.into_text().unwrap(),
                //             //     // ))
                //             // );
                //             // ListItem::new(m.into_text().unwrap())
                //             m.into_text().unwrap()
                //         })
                //         .collect()
            })
            .unwrap_or_default();
        messages.reverse();
        let messages = messages.join("");

        // let messages = List::new(messages).block(Block::bordered().title("Messages"));
        let messages = Paragraph::new(messages.into_text().unwrap())
            .block(Block::bordered().title("Messages"));
        frame.render_widget(messages, messages_area);
    }
}

// Returns the time of a single pulse in microseconds
fn bpq_time(tempo_time: Arc<RwLock<u32>>) -> u32 {
    tempo_time
        .read()
        .map(|bpm| {
            // info!("{bpm}");
            *bpm.deref()
        })
        .unwrap_or(DEFAULT_BPM)
        / BPQ
}

pub fn main() {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or(EnvFilter::new("info"));
    FmtSubscriber::builder()
        .with_file(true)
        .with_line_number(true)
        .with_level(true)
        .with_thread_names(false)
        .with_thread_ids(false)
        .with_env_filter(env_filter)
        .without_time()
        .with_writer(std::sync::Mutex::new(
            // DualWriter::new(
            // std::io::stderr(),
            LogAppender,
            // )
        ))
        .init();

    let (client, _status) = loop {
        if let Ok(midi_out) = jack::Client::new(SYNC_DEV_NAME, jack::ClientOptions::default()) {
            break midi_out;
        }
    };
    let mut sync_pulse_sender = client
        .register_port(SYNC_DEV_PORT_NAME, jack::MidiOut::default())
        .expect("failed to register sync pulse sender");
    let in_port = client
        .register_port(TEMPO_SET_PORT, jack::MidiIn::default())
        .expect("failed to register tempo setter");

    let tempo = Arc::new(RwLock::new(tempo_from_bpm()));

    let mut time_to_send = 0;
    // let mut counter = 0;
    let counter = COUNTER.clone();
    let mut messages: Vec<(u32, Vec<u8>)> = Vec::new();
    debug!("tempo: {:?}", tempo.clone().read());
    debug!("micros: {}", bpq_time(tempo.clone()));
    debug!(
        "time (in frames): {}",
        client.time_to_frames(bpq_time(tempo.clone()) as u64)
    );
    debug!("time_to_send: {}", time_to_send);

    let callback = move |c: &jack::Client, ps: &jack::ProcessScope| -> jack::Control {
        let show_p = in_port.iter(ps);

        for raw_mesg in show_p {
            if let (
                Ok((
                    MidiMsg::Meta {
                        msg: Meta::SetTempo(new_tempo),
                    },
                    _,
                )),
                Ok(mut tempo),
            ) = (MidiMsg::from_midi(raw_mesg.bytes), tempo.write())
            {
                *tempo = (1_000_000 * 60) / new_tempo;
                _ = LOGS
                    .write()
                    .map(|mut logs| logs.push(format!("tempo change: {tempo}")));
            }
        }

        let cycle_times = ps.cycle_times().unwrap();

        if time_to_send == 0 {
            time_to_send = c.time_to_frames(c.time());
            // debug!("time_to_send: {time_to_send}");
        }
        let mut sender = sync_pulse_sender.writer(ps);

        messages.retain_mut(|(time, _msg)| *time >= ps.n_frames());

        while c.time_to_frames(cycle_times.next_usecs) > time_to_send {
            let last_time_send = time_to_send;
            time_to_send = time_to_send.wrapping_add(bpq_time(tempo.clone()));

            let time = c
                .time_to_frames(time_to_send as u64)
                .wrapping_sub(c.time_to_frames(last_time_send as u64));
            let mut bytes = vec![
                MidiMsg::SystemRealTime {
                    msg: SystemRealTimeMsg::TimingClock,
                }
                .to_midi(),
            ];

            // trace!("time: {time}");
            if counter
                .read()
                .is_ok_and(|counter| (counter.deref() % (BPQ * 4)) == 0)
            {
                bytes.push(
                    MidiMsg::SystemRealTime {
                        msg: SystemRealTimeMsg::Start,
                    }
                    .to_midi(),
                );
                info!(
                    "new bar triggered. counter is {}",
                    counter
                        .read()
                        .map(|counter| format!("{counter}"))
                        .unwrap_or("???".into())
                );
            }

            _ = counter.write().map(|mut counter| *counter += 1);

            for bytes in bytes {
                if let Err(e) = sender.write(&jack::RawMidi {
                    time,
                    bytes: &bytes,
                }) {
                    error!("attempt to send sync pulse failed with error: {e}")
                }
            }
        }

        jack::Control::Continue
    };

    // Activate
    let active_client = client
        .activate_async((), jack::contrib::ClosureProcessHandler::new(callback))
        .unwrap();

    // Wait
    panic::set_hook(Box::new(|info| {
        crossterm::execute!(std::io::stderr(), crossterm::terminal::LeaveAlternateScreen)
            .expect("Failed to leave alternate screen");
        eprintln!("Panic: {info:?}");
    }));

    // if color_eyre::install().is_ok() {
    if let Err(e) = color_eyre::install() {
        error!("running TUI failed with error, \"{e}\"");
    } else if let Err(e) = ratatui::run(|terminal| App::new().run(terminal)) {
        error!("{e}");
        println!("{}", LOGS.read().unwrap().join("\n"));
    }
    // }

    // Optional deactivation.
    if let Err(err) = active_client.deactivate() {
        error!("JACK exited with error: {err}");
    };
}
