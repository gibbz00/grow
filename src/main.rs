// mod render;
mod file_watcher;
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
    let (cmd_sender_0, command_reciever) = mpsc::channel();
    let cmd_sender_1 = cmd_sender_0.clone();
    thread::spawn(|| keyevent_handler::run_keyevent_loop(cmd_sender_0));
    thread::spawn(|| file_watcher::watch(cmd_sender_1));
    loop {
        match command_reciever
            .recv()
            .expect("Expected an open keyevent_handler thread.")
        {
            Command::Close => {
                application.close()?;
                break;
            }
            Command::Reload => {
                // TEMP:
                panic!("File updated :)")
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

pub enum Command {
    Close,
    Reload,
}
