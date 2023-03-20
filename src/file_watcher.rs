use crate::{application::UpdateView, send_command, send_error_command, Command};
use notify::{
    event::{AccessKind, AccessMode, RemoveKind},
    EventKind, RecommendedWatcher, Watcher,
};
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
                        Ok(notify_event) => match notify_event.kind {
                            EventKind::Remove(RemoveKind::File) => {
                                send_command!(
                                    cmd_sender,
                                    Command::Update(UpdateView::Remove(
                                        notify_event
                                            .paths
                                            .last()
                                            .expect("Should always return at least one path.")
                                            .clone()
                                    ))
                                )
                            }
                            EventKind::Access(AccessKind::Close(AccessMode::Write)) => {
                                send_command!(
                                    cmd_sender,
                                    Command::Update(UpdateView::Reload(
                                        notify_event
                                            .paths
                                            .last()
                                            .expect("Should always return at least one path.")
                                            .clone()
                                    ))
                                )
                            }
                            _ => {
                                // // TEMP:
                                // send_command!(
                                //     cmd_sender,
                                //     Command::Debug(format!("{:#?}", notify_event))
                                // )
                            }
                        },
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
