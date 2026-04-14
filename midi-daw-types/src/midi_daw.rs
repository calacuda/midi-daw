use clap::{Parser, Subcommand};
use log::error;

mod midi_daw_sync;
mod tui;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[clap(long_about = "starts the midi sync process.")]
    Sync,
    #[clap(
        visible_alias = "run",
        long_about = "plays the passed files (default if no subcmd is present)."
    )]
    Play { files: Vec<String> },
}

fn help_and_exit() -> ! {
    use clap::CommandFactory;
    let mut cmd = Cli::command();
    cmd.print_help().expect("could not print");
    std::process::exit(1)
}

fn main() {
    let cli = Cli::try_parse().unwrap_or_else(|_| {
        let mut a: Vec<String> = std::env::args().collect();

        if a.len() < 2 {
            help_and_exit()
        }

        a.insert(1, "play".into());

        Cli::parse_from(a)
    });

    match cli.command {
        Commands::Sync => midi_daw_sync::main(),
        Commands::Play { files } => {
            if let Err(e) = tui::run(files) {
                error!("failed to run tui. got error: {e}");
            }
        }
    }
}
