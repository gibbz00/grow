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

use crate::Command;

pub struct ClosedApplication;
impl ClosedApplication {
    pub fn open(file_paths: Vec<PathBuf>) -> Result<OpenedApplication> {
        enable_raw_mode()?;
        execute!(io::stdout(), EnterAlternateScreen, Hide)?;
        let application = OpenedApplication {
            focused_view_idx: 0,
            file_paths,
        };
        application.render_view()?;
        Ok(application)
    }
}

pub struct OpenedApplication {
    focused_view_idx: usize,
    file_paths: Vec<PathBuf>,
}

impl OpenedApplication {
    pub fn select_next_view(&mut self) -> Result<()> {
        if self.focused_view_idx != self.file_paths.len() - 1 {
            self.focused_view_idx += 1;
        }
        self.render_view()?;
        Ok(())
    }

    pub fn select_prev_view(&mut self) -> Result<()> {
        self.focused_view_idx = self.focused_view_idx.saturating_sub(1);
        self.render_view()?;
        Ok(())
    }

    pub fn update_view(&mut self, update: UpdateView) -> Result<Option<Command>> {
        match update {
            UpdateView::Remove(file_paths) => {
                for removed_path in file_paths {
                    self.file_paths.remove(self.get_view_index(removed_path));
                    if self.file_paths.is_empty() {
                        return Ok(Some(Command::Close));
                    } else if self.focused_view_idx == self.file_paths.len() {
                        self.focused_view_idx = self.focused_view_idx.saturating_sub(1);
                    }
                }
            }
            UpdateView::Reload(file_paths) => {
                for updated_path in file_paths {
                    self.focused_view_idx = self.get_view_index(updated_path)
                }
            }
        }
        self.render_view()?;
        Ok(None)
    }

    /// Panics if path does not exist in buffer views
    fn get_view_index(&self, file_path: PathBuf) -> usize {
        self.file_paths
            .iter()
            .position(|self_paths| file_path == *self_paths)
            .expect("File path with update exists in application tabs.")
    }

    fn render_view(&self) -> Result<()> {
        execute!(io::stdout(), Clear(crossterm::terminal::ClearType::All))?;
        self.render_tabline()?;
        execute!(
            io::stdout(),
            Print(fs::read_to_string(
                self.file_paths[self.focused_view_idx].clone()
            )?)
        )?;
        Ok(())
    }

    fn render_tabline(&self) -> Result<()> {
        let mut stdout = io::stdout();
        stdout.execute(MoveTo(0, 0))?;

        for file_path in &self.file_paths {
            let absolute_file_path = file_path;
            let tab = format!(
                " {} ",
                absolute_file_path
                    .file_name()
                    .expect("Path to be a valid file name")
                    .to_string_lossy()
            )
            .with(
                if *absolute_file_path == self.file_paths[self.focused_view_idx] {
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

pub enum UpdateView {
    Remove(Vec<PathBuf>),
    Reload(Vec<PathBuf>),
}
