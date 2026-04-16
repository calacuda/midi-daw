use std::{
    fmt,
    fs::remove_file,
    io::{self, BufWriter, Read, Write},
    process,
    sync::{
        Arc, RwLock,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

use bytes::Bytes;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use nix::sys::stat::Mode;
use nix::unistd;
use portable_pty::{CommandBuilder, MasterPty, PtySize, native_pty_system};
use ratatui::{
    DefaultTerminal,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders},
};
use tokio::{
    sync::mpsc::{Sender, channel},
    task::spawn_blocking,
};
use tui_term::{
    vt100,
    widget::{Cursor, PseudoTerminal},
};

#[derive(Debug, Clone, Copy)]
struct Size {
    cols: u16,
    rows: u16,
}

fn round(num: f32) -> u16 {
    // if num.fract() == 0.5 {
    //     num.floor() as u16
    // } else {
    num.round() as u16
    // }
}
async fn run_smux(terminal: &mut DefaultTerminal, files: Vec<String>) -> io::Result<()> {
    let mut size = Size {
        rows: terminal.size()?.height,
        cols: terminal.size()?.width,
    };

    let cwd = std::env::current_dir().unwrap();
    // println!("cwd :  {cwd:?}");
    let pid = process::id();
    // mk named pipe
    let pipe_path = format!("/tmp/midi-daw-{pid}.pipe");
    // Create the FIFO with 0644 permissions
    unistd::mkfifo(pipe_path.as_str(), Mode::from_bits(0o644).unwrap())?;

    // run jurigged in a subprocess with the "JURIDGED_CODE_REDIRECT" env var set to the path of
    // the named pipe
    // let args: Vec<String> = env::args().collect();
    // let args = args[1..].to_vec();
    // let args = args.join(" ");

    // let mut py_cmd = CommandBuilder::new("zsh");
    // py_cmd.arg("-c");
    // py_cmd.arg(format!("cd \"{cwd:?}\" && jurigged -i {}", files.join(" ")));

    let mut py_cmd = CommandBuilder::new("jurigged");
    py_cmd.arg("-i");
    // py_cmd.args(args);
    py_cmd.args(files);
    py_cmd.env("JURIGGGED_CODE_REDIRECT", pipe_path.clone());
    py_cmd.env("PROMPT_TOOLKIT_NO_CPR", "1");
    py_cmd.cwd(cwd);

    // let mut log_cmd = CommandBuilder::new("zsh");
    // log_cmd.arg("-c");
    // log_cmd.arg(format!("cat {pipe_path}"));
    let mut log_cmd = CommandBuilder::new("cat");
    log_cmd.arg(pipe_path.clone());

    let mut panes: Vec<PtyPane> = Vec::new();

    {
        let mut pane_size = size;
        // let pane_size = calc_pane_size(size, 2);
        // let pane_size = ;
        // pane_size.cols -= 2;
        // pane_size.cols *= 6;
        // pane_size.cols /= 10;
        let cols = (pane_size.cols) as f32;
        pane_size.cols = round(cols * (6.5 / 10.));
        open_new_pane(&mut panes, &py_cmd, pane_size)?;
    }
    {
        let mut pane_size = size;
        // let pane_size = calc_pane_size(size, 2);
        // let pane_size = ;
        // pane_size.cols -= 2;
        // pane_size.cols *= 4;
        // pane_size.cols /= 10;
        let cols = (pane_size.cols) as f32;
        pane_size.cols -= round(cols * (6.5 / 10.));
        open_new_pane(&mut panes, &log_cmd, pane_size)?;
    }
    let active_pane = Some(0);

    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .margin(1)
                .constraints([Constraint::Percentage(65), Constraint::Percentage(35)].as_ref())
                .split(f.area());

            for (index, pane) in panes.iter().enumerate() {
                let block = Block::default()
                    .borders(Borders::ALL)
                    .style(Style::default().add_modifier(Modifier::BOLD));
                let mut cursor = Cursor::default();
                let block = if Some(index) == active_pane {
                    block.style(
                        Style::default()
                            .add_modifier(Modifier::BOLD)
                            .fg(Color::LightMagenta),
                    )
                } else {
                    cursor.hide();
                    block
                };
                let parser = pane.parser.read().unwrap();
                let screen = parser.screen();
                let pseudo_term = PseudoTerminal::new(screen).block(block).cursor(cursor);

                f.render_widget(pseudo_term, chunks[index]);
            }
        })?;

        if event::poll(Duration::from_millis(10))? {
            tracing::info!("Terminal Size: {:?}", terminal.size());
            match event::read()? {
                Event::Key(key) => match key.code {
                    KeyCode::Char('q') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        remove_file(pipe_path)?;
                        return Ok(());
                    }
                    // KeyCode::Char('q') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    //     return Ok(());
                    // }
                    // KeyCode::Char('n') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    //     let pane_size = calc_pane_size(size, panes.len() + 1);
                    //     tracing::info!("Opened new pane with size: {size:?}");
                    //     resize_all_panes(&mut panes, pane_size);
                    //     open_new_pane(&mut panes, &mut active_pane, &cmd, pane_size)?;
                    // }
                    // KeyCode::Char('x') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    //     close_active_pane(&mut panes, &mut active_pane).await?;
                    //     resize_all_panes(&mut panes, pane_size);
                    // }
                    // KeyCode::Char('k') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    //     if let Some(pane) = active_pane {
                    //         active_pane = Some(pane.saturating_sub(1));
                    //     }
                    // }
                    // KeyCode::Char('j') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    //     if let Some(pane) = active_pane {
                    //         if pane < panes.len() - 1 {
                    //             active_pane = Some(pane.saturating_add(1));
                    //         }
                    //     }
                    // }
                    _ => {
                        if let Some(index) = active_pane
                            && handle_pane_key_event(&mut panes[index], &key).await
                        {
                            continue;
                        }
                    } // }
                },
                Event::Resize(cols, rows) => {
                    tracing::info!("Resized to: rows: {} cols: {}", rows, cols);
                    size.rows = rows;
                    size.cols = cols;
                    let pane_size = calc_pane_size(size, panes.len());
                    resize_all_panes(&mut panes, pane_size);
                }
                _ => {}
            }
        }

        // cleanup_exited_panes(&mut panes, &mut active_pane);

        if panes.is_empty() {
            remove_file(pipe_path)?;
            return Ok(());
        }
    }
}

fn calc_pane_size(mut size: Size, nr_panes: usize) -> Size {
    size.cols -= 2;
    size.cols /= nr_panes as u16;
    size
}

fn resize_all_panes(panes: &mut [PtyPane], size: Size) {
    for pane in panes.iter() {
        pane.resize(size);
    }
}

struct PtyPane {
    parser: Arc<RwLock<vt100::Parser>>,
    sender: Sender<Bytes>,
    master_pty: Box<dyn MasterPty>,
    // exited: Arc<AtomicBool>,
}

impl PtyPane {
    fn new(size: Size, cmd: CommandBuilder) -> io::Result<Self> {
        let pty_system = native_pty_system();
        let pty_pair = pty_system
            .openpty(PtySize {
                rows: size.rows - 4,
                cols: size.cols - 4,
                pixel_width: 0,
                pixel_height: 0,
            })
            .unwrap();
        let parser = Arc::new(RwLock::new(vt100::Parser::new(
            size.rows - 4,
            size.cols - 4,
            0,
        )));
        let exited = Arc::new(AtomicBool::new(false));

        {
            let exited_clone = exited.clone();
            spawn_blocking(move || {
                let mut child = pty_pair.slave.spawn_command(cmd).unwrap();
                let _ = child.wait();
                exited_clone.store(true, Ordering::Relaxed);
                drop(pty_pair.slave);
            });
        }

        {
            let mut reader = pty_pair.master.try_clone_reader().unwrap();
            let parser = parser.clone();
            tokio::spawn(async move {
                let mut processed_buf = Vec::new();
                let mut buf = [0u8; 8192];

                loop {
                    let size = reader.read(&mut buf).unwrap();
                    if size == 0 {
                        break;
                    }
                    if size > 0 {
                        processed_buf.extend_from_slice(&buf[..size]);
                        let mut parser = parser.write().unwrap();
                        parser.process(&processed_buf);

                        // Clear the processed portion of the buffer
                        processed_buf.clear();
                    }
                }
            });
        }

        let (tx, mut rx) = channel::<Bytes>(32);

        let mut writer = BufWriter::new(pty_pair.master.take_writer().unwrap());
        // writer is moved into the tokio task below
        tokio::spawn(async move {
            while let Some(bytes) = rx.recv().await {
                writer.write_all(&bytes).unwrap();
                writer.flush().unwrap();
            }
        });

        Ok(Self {
            parser,
            sender: tx,
            master_pty: pty_pair.master,
            // exited,
        })
    }

    fn resize(&self, size: Size) {
        self.parser
            .write()
            .unwrap()
            .screen_mut()
            .set_size(size.rows - 4, size.cols - 4);
        self.master_pty
            .resize(PtySize {
                rows: size.rows - 4,
                cols: size.cols - 4,
                pixel_width: 0,
                pixel_height: 0,
            })
            .unwrap();
    }

    // fn is_alive(&self) -> bool {
    //     !self.exited.load(Ordering::Relaxed)
    // }
}

async fn handle_pane_key_event(pane: &mut PtyPane, key: &KeyEvent) -> bool {
    let input_bytes = match key.code {
        KeyCode::Char(ch) => {
            let mut send = vec![ch as u8];
            let upper = ch.to_ascii_uppercase();
            if key.modifiers == KeyModifiers::CONTROL {
                match upper {
                    'N' => {
                        // Ignore Ctrl+n within a pane
                        return true;
                    }
                    'X' => {
                        // Close the pane
                        return false;
                    }
                    // https://github.com/fyne-io/terminal/blob/master/input.go
                    // https://gist.github.com/ConnerWill/d4b6c776b509add763e17f9f113fd25b
                    '2' | '@' | ' ' => send = vec![0],
                    '3' | '[' => send = vec![27],
                    '4' | '\\' => send = vec![28],
                    '5' | ']' => send = vec![29],
                    '6' | '^' => send = vec![30],
                    '7' | '-' | '_' => send = vec![31],
                    char if ('A'..='_').contains(&char) => {
                        // Since A == 65,
                        // we can safely subtract 64 to get
                        // the corresponding control character
                        let ascii_val = char as u8;
                        let ascii_to_send = ascii_val - 64;
                        send = vec![ascii_to_send];
                    }
                    _ => {}
                }
            }
            send
        }
        #[cfg(unix)]
        KeyCode::Enter => vec![b'\n'],
        #[cfg(windows)]
        KeyCode::Enter => vec![b'\r', b'\n'],
        KeyCode::Backspace => vec![8],
        KeyCode::Left => vec![27, 91, 68],
        KeyCode::Right => vec![27, 91, 67],
        KeyCode::Up => vec![27, 91, 65],
        KeyCode::Down => vec![27, 91, 66],
        KeyCode::Tab => vec![9],
        KeyCode::Home => vec![27, 91, 72],
        KeyCode::End => vec![27, 91, 70],
        KeyCode::PageUp => vec![27, 91, 53, 126],
        KeyCode::PageDown => vec![27, 91, 54, 126],
        KeyCode::BackTab => vec![27, 91, 90],
        KeyCode::Delete => vec![27, 91, 51, 126],
        KeyCode::Insert => vec![27, 91, 50, 126],
        KeyCode::Esc => vec![27],
        _ => return true,
    };

    pane.sender.send(Bytes::from(input_bytes)).await.ok();
    true
}

fn open_new_pane(panes: &mut Vec<PtyPane>, cmd: &CommandBuilder, size: Size) -> io::Result<()> {
    let new_pane = PtyPane::new(size, cmd.clone())?;
    panes.push(new_pane);
    Ok(())
}

impl fmt::Debug for PtyPane {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let parser = self.parser.read().unwrap();
        let screen = parser.screen();

        f.debug_struct("PtyPane").field("screen", screen).finish()
    }
}

#[tokio::main]
pub async fn run(files: Vec<String>) -> io::Result<()> {
    // spin up ratatui with the ability to read the output of jurigged and the script
    let mut terminal = ratatui::init();
    let result = run_smux(&mut terminal, files).await;
    ratatui::restore();

    result
}
