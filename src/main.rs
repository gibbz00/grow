// mod render;
mod file_watcher;
mod keyevent_handler;

use anyhow::anyhow;
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
    sync::mpsc,
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

fn run_application() -> anyhow::Result<()> {
    let application: Application<Opened> = Application::open()?;

    // TODO: close application on error.
    let (cmd_sender_0, command_reciever) = mpsc::channel();
    let cmd_sender_1 = cmd_sender_0.clone();
    let found_thread_0 = thread::Builder::new()
        .name("keyevent_handler".into())
        .spawn(|| keyevent_loop(cmd_sender_0));
    if let Err(error) = found_thread_0 {
        application.close()?;
        return Err(anyhow!(error));
    }
    let found_thread_1 = thread::Builder::new()
        .name("file_watcher".into())
        .spawn(|| file_watcher(cmd_sender_1));
    if let Err(error) = found_thread_1 {
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
            Err(err_msg) => {
                application.close()?;
                return Err(err_msg);
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
