// mod render;
mod application;
mod args;
mod file_watcher;
mod keyevent_handler;
mod thread_helpers;

use anyhow::{anyhow, Result};
use application::Application;
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
    // parse args
    let args = args::Args::parse();
    for file in &args.files[..] {
        if !file.is_file() {
            return Err(anyhow!("Failed to locate file: {:?}", file));
        }
    }

    let application = Application::open()?;
    application.render_files(&args.files)?;

    let (cmd_sender, command_reciever) = mpsc::channel();
    // TEMP:
    let thread_closures = thread_closures!(
        keyevent_loop,
        filewatcher(args.files.first().unwrap().clone())
    );
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
                // TEMP:
                Command::Reload => application.render_files(&args.files)?,
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
    Reload,
}
