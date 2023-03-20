use crate::{
    thread_helpers::{send_command, send_error_command},
    Command,
};
use crossterm::event::{read, Event as CrosstermEvent, KeyCode::*, KeyModifiers};
use std::sync::mpsc::Sender;

pub fn keyevent_loop(cmd_sender: Sender<anyhow::Result<Command>>) {
    loop {
        match read() {
            Ok(crossterm_event) => {
                if let CrosstermEvent::Key(keyevent) = crossterm_event {
                    match (keyevent.modifiers, keyevent.code) {
                        (KeyModifiers::NONE, Char(ch)) if ch == '<' => {
                            send_command(&cmd_sender, Command::PrevView);
                        }
                        (KeyModifiers::NONE, Char(ch)) if ch == '>' => {
                            send_command(&cmd_sender, Command::NextView);
                        }
                        (KeyModifiers::CONTROL, Char(ch)) if ch == 'c' => {
                            send_command(&cmd_sender, Command::Close);
                        }
                        (_, _) => (),
                    }
                }
            }
            Err(error) => send_error_command(&cmd_sender, error),
        }
    }
}
