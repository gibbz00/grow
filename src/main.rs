// mod render;
mod keyevent_handler;

use crossterm::{
    cursor::{Hide, Show},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{
    io::{self, Write},
    process::ExitCode,
    sync::mpsc,
    thread,
};

fn main() -> ExitCode {
    match run_application() {
        Ok(_) => ExitCode::SUCCESS,
        Err(message) => {
            std::io::stderr()
                .write_all(format!("{message}").as_bytes())
                .expect("Expected write access to stderr.");
            ExitCode::FAILURE
        }
    }
}

fn run_application() -> io::Result<()> {
    let application: Application<Opened> = Application::open()?;
    let (command_sender, command_reciever) = mpsc::channel();
    thread::spawn(|| keyevent_handler::run_keyevent_loop(command_sender));
    loop {
        match command_reciever
            .recv()
            .expect("Expected an open keyevent_handler thread.")
        {
            Commands::Close => {
                application.close()?;
                break;
            }
            // TEMP:
            Commands::Stub => {
                continue;
            }
        }
    }
    // let markdown_string = fs::read_to_string(Path::new("README.md")).unwrap();
    // execute!(stdout, Print(markdown_string))?;

    Ok(())
}

struct Application<State> {
    state: std::marker::PhantomData<State>,
}

struct Closed;
impl Application<Closed> {
    pub fn open() -> io::Result<Application<Opened>> {
        enable_raw_mode()?;
        execute!(io::stdout(), EnterAlternateScreen, Hide)?;
        Ok(Application {
            state: std::marker::PhantomData,
        })
    }
}

struct Opened;
impl Application<Opened> {
    pub fn close(self) -> io::Result<Application<Closed>> {
        disable_raw_mode()?;
        execute!(io::stdout(), LeaveAlternateScreen, Show)?;
        Ok(Application {
            state: std::marker::PhantomData,
        })
    }
}

pub enum Commands {
    Close,
    Stub,
}
