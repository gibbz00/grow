use anyhow::Result;
use crossterm::{
    cursor::{Hide, MoveTo, MoveToNextLine, Show},
    execute,
    style::Print,
    terminal::{
        disable_raw_mode, enable_raw_mode, Clear, EnterAlternateScreen, LeaveAlternateScreen,
    },
};
use std::{
    fs, io,
    path::{Path, PathBuf},
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
    pub fn render_file(&self, file_path: &PathBuf) -> Result<()> {
        execute!(io::stdout(), Clear(crossterm::terminal::ClearType::All))?;
        render_tabline(file_path)?;
        execute!(io::stdout(), Print(fs::read_to_string(file_path)?))?;
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

pub fn render_tabline(file_path: &Path) -> Result<()> {
    execute!(
        io::stdout(),
        MoveTo(0, 0),
        Print(file_path.to_string_lossy()),
        MoveToNextLine(1)
    )?;
    Ok(())
}
