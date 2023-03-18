use std::sync::mpsc::Sender;

use crate::Command;
use crossterm::event::{read, Event as CrosstermEvent, KeyCode, KeyModifiers};

pub fn run_keyevent_loop(sender: Sender<Command>) {
    loop {
        if let CrosstermEvent::Key(keyevent) = read().expect("Expected read access to input.") {
            match (keyevent.modifiers, keyevent.code) {
                (KeyModifiers::CONTROL, KeyCode::Char(character)) if character == 'c' => {
                    sender
                        .send(Command::Close)
                        .expect("Expected an active main thread.");
                }
                (_, _) => (),
            }
        }
    }
}
