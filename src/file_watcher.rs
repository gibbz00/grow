use std::{path::Path, sync::mpsc::Sender};

use notify::Watcher;

use crate::Command;

pub fn watch(command_sender: Sender<Command>) {
    let mut watcher = notify::recommended_watcher(move |response| match response {
        // TEMP: unwrap
        Ok(_notify_event) => command_sender.send(Command::Reload).unwrap(),
        Err(message) => panic!("{message}"),
    })
    .expect("Expected file watching access.");

    // NOTE: recursive mode irrelevant.
    // TODO: specify file, check existence
    watcher
        .watch(Path::new("test.md"), notify::RecursiveMode::NonRecursive)
        .expect("Watch access to FILE");
}
