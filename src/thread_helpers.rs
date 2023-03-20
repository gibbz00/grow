#[macro_export]
macro_rules! thread_closures {
    ($($thread_closure:expr),*) => {
        {
            let thread_closures: $crate::thread_helpers::ThreadClosures = vec![$(Box::new($thread_closure)),*];
            thread_closures
        }
    };
}

use crate::Command;
use anyhow::Result;
use std::{sync::mpsc::Sender, thread};

pub fn send_command(cmd_sender: &Sender<Result<Command>>, command: Command) {
    // Unwrapping as thread errors should not normally be handled by user.
    (*cmd_sender).send(Ok(command)).unwrap();
}

pub fn send_error_command<E>(cmd_sender: &Sender<Result<Command>>, error: E)
where
    E: std::convert::Into<anyhow::Error>,
{
    // Unwrapping as thread errors should not normally be handled by user.
    (*cmd_sender).send(Err(anyhow::anyhow!(error))).unwrap();
}

// NOTE: once trait aliases are introcuded.
// trait CommandSender = FnOnce(Sender<Result<Command>>) + Send + 'static;
pub type ThreadClosures = Vec<Box<dyn FnOnce(Sender<Result<Command>>) + Send + 'static>>;
pub fn spawn_threads(
    cmd_sender: Sender<Result<Command>>,
    thread_closures: ThreadClosures,
) -> Result<()> {
    // NOTE: cmd_sender is cloned once more than needed
    for thread_closure in thread_closures {
        let cmd_sender_clone = cmd_sender.clone();
        spawn_thread(cmd_sender_clone, thread_closure)?;
    }

    Ok(())
}

fn spawn_thread<F>(
    cmd_sender: Sender<Result<Command>>,
    closure: F,
) -> std::io::Result<thread::JoinHandle<()>>
where
    F: FnOnce(Sender<Result<Command>>) + Send + 'static,
{
    // Using builder as it might return error on spawn.
    // thread::spawn() simply panics.
    thread::Builder::new().spawn(|| closure(cmd_sender))
}
