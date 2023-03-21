use crate::{
    thread_helpers::{send_command, send_error_command},
    Command,
};
use crossterm::event::{read, Event as CrosstermEvent, KeyCode::*, KeyModifiers};
use std::sync::mpsc::Sender;

pub fn event_loop(cmd_sender: Sender<anyhow::Result<Command>>) {
    loop {
        match read() {
            Ok(crossterm_event) => match crossterm_event {
                CrosstermEvent::Resize(_, _) => {
                    send_command(&cmd_sender, Command::AutoResize);
                }
                CrosstermEvent::Key(keyevent) => match (keyevent.modifiers, keyevent.code) {
                    (KeyModifiers::NONE, Char(ch)) if ch == 'd' => {
                        send_command(&cmd_sender, Command::Scroll(1));
                    }
                    (KeyModifiers::NONE, Char(ch)) if ch == 'u' => {
                        send_command(&cmd_sender, Command::Scroll(-1));
                    }
                    (KeyModifiers::NONE, Char(ch)) if ch == '<' => {
                        send_command(&cmd_sender, Command::PrevView);
                    }
                    (KeyModifiers::NONE, Char(ch)) if ch == '>' => {
                        send_command(&cmd_sender, Command::NextView);
                    }
                    (KeyModifiers::CONTROL, Char(ch)) if ch == 'c' => {
                        send_command(&cmd_sender, Command::Close);
                    }
                    _ => (),
                },
                _ => (),
            },
            Err(error) => send_error_command(&cmd_sender, error),
        }
    }
}
