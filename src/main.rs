// mod render;
mod application;
mod args;
mod file_watcher;
mod keyevent_handler;
mod thread_helpers;

use anyhow::{anyhow, Result};
use application::ClosedApplication;
use clap::Parser;
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
    let args = args::Args::parse();
    for file in &args.files[..] {
        if !file.is_file() {
            return Err(anyhow!("Failed to locate file: {:?}", file));
        }
    }

    let mut application = ClosedApplication::open(&args.files)?;

    let (cmd_sender, command_reciever) = mpsc::channel();
    let thread_closures = thread_closures!(keyevent_loop, filewatcher(args.files));
    if let Err(error) = thread_helpers::spawn_threads(cmd_sender, thread_closures) {
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
                Command::NextView => application.select_next_view()?,
                Command::PrevView => application.select_prev_view()?,
                Command::Update(update) => {
                    let found_command_response = application.update_view(update)?;
                    if let Some(Command::Close) = found_command_response {
                        application.close()?;
                        println!("All opened files have been removed. Closing application.");
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
