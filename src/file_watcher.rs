use crate::Command;
use anyhow::anyhow;
use notify::{RecommendedWatcher, Watcher};
use std::{
    path::Path,
    sync::mpsc::{self, Sender},
};

pub fn file_watcher(cmd_sender: Sender<anyhow::Result<Command>>) {
    let (file_change_sender, file_change_reciever) = mpsc::channel();
    let found_watcher = RecommendedWatcher::new(file_change_sender, notify::Config::default());
    match found_watcher {
        Ok(mut watcher) => {
            // TODO: specify file(s)
            // NOTE: recursive mode irrelevant for single file watch.
            if let Err(err) =
                watcher.watch(Path::new("README.md"), notify::RecursiveMode::NonRecursive)
            {
                return cmd_sender.send(Err(anyhow!(err))).unwrap();
            }

            for response in file_change_reciever {
                match response {
                    // TEMP: unwrap
                    Ok(_notify_event) => cmd_sender.send(Ok(Command::Reload)).unwrap(),
                    Err(err) => cmd_sender.send(Err(anyhow!(err))).unwrap(),
                }
            }
        }
        Err(err) => {
            cmd_sender.send(Err(anyhow!(err))).unwrap();
        }
    }
}
