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
    pub fn open(file_paths: &[PathBuf]) -> Result<OpenedApplication> {
        enable_raw_mode()?;
        execute!(io::stdout(), EnterAlternateScreen, Hide)?;
        let application = OpenedApplication {
            focused_file_idx: 0,
            file_paths,
        };
        application.render_view()?;
        Ok(application)
    }
}

pub struct OpenedApplication<'a> {
    focused_file_idx: usize,
    file_paths: &'a [PathBuf],
}

impl OpenedApplication<'_> {
    pub fn next_file(&mut self) -> Result<()> {
        if self.focused_file_idx != self.file_paths.len() - 1 {
            self.focused_file_idx += 1;
        }
        self.render_view()?;
        Ok(())
    }

    pub fn prev_file(&mut self) -> Result<()> {
        self.focused_file_idx = self.focused_file_idx.saturating_sub(1);
        self.render_view()?;
        Ok(())
    }

    pub fn reload(&mut self, file_path_with_update: PathBuf) -> Result<()> {
        self.focused_file_idx = self
            .file_paths
            .iter()
            .position(|file_path| {
                let absolute_path = fs::canonicalize(file_path)
                    .expect("File path existence check done when parsing args.");
                absolute_path == file_path_with_update
            })
            .expect("File path with update exists in application tabs.");
        self.render_view()?;
        Ok(())
    }

    fn render_view(&self) -> Result<()> {
        execute!(io::stdout(), Clear(crossterm::terminal::ClearType::All))?;
        self.render_tabline()?;
        execute!(
            io::stdout(),
            Print(fs::read_to_string(
                self.file_paths[self.focused_file_idx].clone()
            )?)
        )?;
        Ok(())
    }

    fn render_tabline(&self) -> Result<()> {
        let mut stdout = io::stdout();
        stdout.execute(MoveTo(0, 0))?;

        for file_path in self.file_paths {
            let tab = format!(" {} ", file_path.to_string_lossy()).with(
                if file_path == &self.file_paths[self.focused_file_idx] {
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
