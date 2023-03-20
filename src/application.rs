use anyhow::Result;
use crossterm::{
    cursor::{Hide, MoveTo, MoveToNextLine, Show},
    execute,
    style::{Color, Print, PrintStyledContent, Stylize},
    terminal::{
        disable_raw_mode, enable_raw_mode, Clear, EnterAlternateScreen, LeaveAlternateScreen,
    },
    ExecutableCommand,
};
use std::{fs, io, path::PathBuf};

pub struct ClosedApplication;
impl ClosedApplication {
    pub fn open() -> io::Result<OpenedApplication> {
        enable_raw_mode()?;
        execute!(io::stdout(), EnterAlternateScreen, Hide)?;
        Ok(OpenedApplication::default())
    }
}

#[derive(Default)]
pub struct OpenedApplication {
    focused_file_idx: usize,
}

impl OpenedApplication {
    pub fn render_files(&self, file_paths: &[PathBuf]) -> Result<()> {
        execute!(io::stdout(), Clear(crossterm::terminal::ClearType::All))?;
        self.render_tabline(file_paths)?;
        // TODO: render active file
        execute!(
            io::stdout(),
            Print(fs::read_to_string(file_paths.first().unwrap())?)
        )?;
        Ok(())
    }

    fn render_tabline(&self, file_paths: &[PathBuf]) -> Result<()> {
        let mut stdout = io::stdout();
        stdout.execute(MoveTo(0, 0))?;

        for file_path in file_paths {
            let tab = format!(" {} ", file_path.to_string_lossy()).with(
                if file_path == &file_paths[self.focused_file_idx] {
                    Color::White
                } else {
                    Color::DarkGrey
                },
            );
            stdout.execute(PrintStyledContent(tab))?;
        }
        stdout.execute(MoveToNextLine(1))?;

        Ok(())
    }

    pub fn close(self) -> io::Result<ClosedApplication> {
        disable_raw_mode()?;
        execute!(io::stdout(), LeaveAlternateScreen, Show)?;
        Ok(ClosedApplication {})
    }
}
