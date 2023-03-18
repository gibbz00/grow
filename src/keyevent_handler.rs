use std::sync::mpsc::Sender;

use crate::Commands;
use crossterm::event::{read, Event as CrosstermEvent, KeyCode, KeyModifiers};

pub fn run_keyevent_loop(sender: Sender<Commands>) {
    loop {
        if let CrosstermEvent::Key(keyevent) = read().expect("Expected read access to input.") {
            match (keyevent.modifiers, keyevent.code) {
                (KeyModifiers::CONTROL, KeyCode::Char(character)) if character == 'c' => {
                    sender
                        .send(Commands::Close)
                        .expect("Expected an active main thread.");
                }
                (_, _) => (),
            }
        }
    }
}
