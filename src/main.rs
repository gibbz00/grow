// mod render;
mod application;
mod args;
mod file_watcher;
mod keyevent_handler;
mod thread_helpers;

use anyhow::Result;
use application::ClosedApplication;
use file_watcher::filewatcher;
use keyevent_handler::keyevent_loop;
use std::{io::Write, process::ExitCode, sync::mpsc};

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
    let args = args::parse()?;
    let (cmd_sender, command_reciever) = mpsc::channel();
    thread_helpers::spawn_threads(
        cmd_sender,
        thread_closures!(keyevent_loop, filewatcher(args.files.clone())),
    )?;
    let mut application = ClosedApplication::open(args.files)?;
    loop {
        match command_reciever.recv().unwrap() {
            Ok(command) => match command {
                Command::Close => {
                    application.close()?;
                    break;
                }
                Command::NextView => application.select_next_view()?,
                Command::PrevView => application.select_prev_view()?,
                Command::Update(update) => {
                    let found_command_response = application.update_view(update)?;
                    if let Some(Command::Close) = found_command_response {
                        application.close()?;
                        println!("All opened files have been (re)moved. Closing application.");
                        return Ok(());
                    }
                }
                // TEMP:
                Command::Debug(msg) => {
                    application.debug(msg)?;
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

pub enum Command {
    Close,
    Update(application::UpdateView),
    NextView,
    PrevView,
    // TEMP:
    Debug(String),
}
