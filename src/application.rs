use std::{fs, io, path::PathBuf};

use crossterm::{
    cursor::{Hide, MoveTo, Show},
    execute,
    style::Print,
    terminal::{
        disable_raw_mode, enable_raw_mode, Clear, EnterAlternateScreen, LeaveAlternateScreen,
    },
};

pub struct Application<State> {
    state: std::marker::PhantomData<State>,
}

pub struct Closed;
impl Application<Closed> {
    pub fn open() -> io::Result<Application<Opened>> {
        enable_raw_mode()?;
        execute!(io::stdout(), EnterAlternateScreen, Hide)?;
        Ok(Application {
            state: std::marker::PhantomData,
        })
    }
}

pub struct Opened;
impl Application<Opened> {
    pub fn render_file(&self, file_path: &PathBuf) -> anyhow::Result<()> {
        execute!(
            io::stdout(),
            MoveTo(0, 0),
            Clear(crossterm::terminal::ClearType::All),
            Print(fs::read_to_string(file_path)?)
        )?;
        Ok(())
    }

    pub fn close(self) -> io::Result<Application<Closed>> {
        disable_raw_mode()?;
        execute!(io::stdout(), LeaveAlternateScreen, Show)?;
        Ok(Application {
            state: std::marker::PhantomData,
        })
    }
}
