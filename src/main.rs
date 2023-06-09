mod application;
mod args;
mod file_watcher;
mod markdown_renderer;
mod term_event_handler;
mod thread_helpers;

use anyhow::Result;
use application::ClosedApplication;
use file_watcher::filewatcher;
use std::{io::Write, process::ExitCode, sync::mpsc};
use term_event_handler::event_loop;

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
        thread_closures!(event_loop, filewatcher(args.files.clone())),
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
                Command::Scroll(steps) => application.scroll_markdown_view(steps)?,
                Command::Update(update) => {
                    let found_command_response = application.update_view(update)?;
                    if let Some(Command::Close) = found_command_response {
                        application.close()?;
                        println!("All opened files have been (re)moved. Closing application.");
                        return Ok(());
                    }
                }
            },
            Err(error) => {
                application.close()?;
                return Err(error);
            }
        }
    }

    Ok(())
}

pub enum Command {
    Close,
    Update(application::UpdateView),
    NextView,
    PrevView,
    Scroll(i16),
}
