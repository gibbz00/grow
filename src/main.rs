// mod render;
mod file_watcher;
mod keyevent_handler;
mod thread_helpers;

use anyhow::{anyhow, Result};
use crossterm::{
    cursor::{Hide, Show},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use file_watcher::file_watcher;
use keyevent_handler::keyevent_loop;
use std::{
    io::{self, Write},
    process::ExitCode,
    sync::mpsc::{self, Sender},
    thread,
};

fn main() -> ExitCode {
    match run_application() {
        Ok(_) => ExitCode::SUCCESS,
        Err(message) => {
            std::io::stderr()
                .write_all(format!("{message}\n").as_bytes())
                .expect("Expected write access to stderr.");
            ExitCode::FAILURE
        }
    }
}

fn run_application() -> Result<()> {
    let application: Application<Opened> = Application::open()?;

    let (cmd_sender, command_reciever) = mpsc::channel();
    if let Err(error) = spawn_threads(cmd_sender) {
        application.close()?;
        return Err(anyhow!(error));
    }

    loop {
        match command_reciever.recv().unwrap() {
            Ok(command) => match command {
                Command::Close => {
                    application.close()?;
                    break;
                }
                Command::Reload => {
                    // TEMP:
                    panic!("File updated :)")
                }
            },
            Err(error) => {
                application.close()?;
                return Err(error);
            }
        }
    }
    // let markdown_string = fs::read_to_string(Path::new("README.md")).unwrap();
    // execute!(stdout, Print(markdown_string))?;

    Ok(())
}

fn spawn_threads(cmd_sender: Sender<Result<Command>>) -> Result<()> {
    let cmd_sender_clone = cmd_sender.clone();
    spawn_thread("keyevent_handler", cmd_sender, keyevent_loop)?;
    spawn_thread("file_watcher", cmd_sender_clone, file_watcher)?;
    Ok(())
}

fn spawn_thread<F>(
    name: &str,
    cmd_sender: Sender<Result<Command>>,
    closure: F,
) -> io::Result<thread::JoinHandle<()>>
where
    F: FnOnce(Sender<Result<Command>>) + Send + 'static,
{
    // Using builder as it returns error instead on spawn.
    // thread::spawn() simply panics.
    thread::Builder::new()
        .name(name.into())
        .spawn(|| closure(cmd_sender))
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
