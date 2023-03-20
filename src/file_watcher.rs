use crate::{
    application::UpdateView,
    thread_helpers::{send_command, send_error_command},
    Command,
};
use anyhow::Result;
use notify::{
    event::{AccessKind, AccessMode, RemoveKind},
    EventKind, RecommendedWatcher, Watcher,
};
use std::{
    path::PathBuf,
    sync::mpsc::{self, Sender},
};

pub fn filewatcher(file_paths: Vec<PathBuf>) -> impl FnOnce(Sender<Result<Command>>) {
    move |cmd_sender: Sender<anyhow::Result<Command>>| {
        let (file_change_sender, file_change_reciever) = mpsc::channel();
        if let Err(watch_error) = RecommendedWatcher::new(
            file_change_sender,
            notify::Config::default(),
        )
        .map(|mut watcher| -> Result<()> {
            for file_path in &file_paths {
                watcher.watch(file_path, notify::RecursiveMode::NonRecursive)?
            }
            let mut file_paths = file_paths;
            loop {
                let notify_event = file_change_reciever.recv()??;
                match notify_event.kind {
                    EventKind::Remove(RemoveKind::File) => {
                        // Multiple file deletions can occur between each notify remove event.
                        // We don't want to attempt to render files which don't exists.
                        // IMPROVEMENT: replace with drain filter once it becomes stable.
                        // let removed_files = file_paths.drain_filter(|file| !file.exists())
                        let mut removed_files: Vec<PathBuf> = Vec::new();
                        for i in (0..file_paths.len()).rev() {
                            if !file_paths[i].exists() {
                                removed_files.push(file_paths.swap_remove(i));
                            }
                        }
                        // Don't rerender the view from a new remove notification when the
                        // file removal was handled in the previous send_command.
                        if !removed_files.is_empty() {
                            send_command(
                                &cmd_sender,
                                Command::Update(UpdateView::Remove(removed_files)),
                            )
                        }
                    }
                    EventKind::Access(AccessKind::Close(AccessMode::Write)) => send_command(
                        &cmd_sender,
                        Command::Update(UpdateView::Reload(notify_event.paths)),
                    ),
                    _ => {
                        // TEMP:
                        // send_command!(
                        //     cmd_sender,
                        //     Command::Debug(format!("gibbz: {:#?}", notify_event))
                        // )
                    }
                }
            }
        }) {
            send_error_command(&cmd_sender, watch_error);
        }
    }
}
