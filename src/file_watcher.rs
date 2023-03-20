use crate::{send_command, send_error_command, Command};
use notify::{RecommendedWatcher, Watcher};
use std::{
    path::PathBuf,
    sync::mpsc::{self, Sender},
};

pub fn filewatcher(file_paths: Vec<PathBuf>) -> impl FnOnce(Sender<anyhow::Result<Command>>) {
    move |cmd_sender: Sender<anyhow::Result<Command>>| {
        let (file_change_sender, file_change_reciever) = mpsc::channel();
        let found_watcher = RecommendedWatcher::new(file_change_sender, notify::Config::default());
        match found_watcher {
            Ok(mut watcher) => {
                for file_path in file_paths {
                    if let Err(error) =
                        // NOTE: recursive mode irrelevant for single file watch.
                        watcher.watch(&file_path, notify::RecursiveMode::NonRecursive)
                    {
                        return send_error_command!(cmd_sender, error);
                    }
                }

                for response in file_change_reciever {
                    match response {
                        Ok(notify_event) => send_command!(
                            cmd_sender,
                            Command::Reload(
                                notify_event
                                    .paths
                                    .last()
                                    .expect("Should always return at least one path.")
                                    .clone()
                            )
                        ),
                        Err(error) => return send_error_command!(cmd_sender, error),
                    }
                }
            }
            Err(error) => {
                send_error_command!(cmd_sender, error);
            }
        }
    }
}
