use anyhow::Result;
use crossterm::{
    cursor::{Hide, MoveTo, MoveToNextLine, Show},
    execute,
    style::Print,
    terminal::{
        disable_raw_mode, enable_raw_mode, Clear, EnterAlternateScreen, LeaveAlternateScreen,
    },
    ExecutableCommand,
};
use std::{fs, io, path::PathBuf};

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
    pub fn render_files(&self, file_paths: &[PathBuf]) -> Result<()> {
        execute!(io::stdout(), Clear(crossterm::terminal::ClearType::All))?;
        render_tabline(file_paths)?;
        // TODO: render active file
        execute!(
            io::stdout(),
            Print(fs::read_to_string(file_paths.first().unwrap())?)
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

pub fn render_tabline(file_paths: &[PathBuf]) -> Result<()> {
    let mut stdout = io::stdout();
    stdout.execute(MoveTo(0, 0))?;

    for file_path in file_paths {
        stdout.execute(Print(format!(" {} ", file_path.to_string_lossy())))?;
    }
    stdout.execute(MoveToNextLine(1))?;

    Ok(())
}
