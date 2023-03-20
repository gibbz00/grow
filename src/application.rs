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
    pub fn open(file_paths: &[PathBuf]) -> Result<OpenedApplication> {
        enable_raw_mode()?;
        execute!(io::stdout(), EnterAlternateScreen, Hide)?;
        let application = OpenedApplication {
            focused_view_idx: 0,
            buffer_views: file_paths
                .iter()
                .map(|file_path| BufferView {
                    absolute_file_path: fs::canonicalize(file_path)
                        .expect("File path existence check done when parsing args."),
                })
                .collect(),
        };
        application.render_view()?;
        Ok(application)
    }
}

pub struct OpenedApplication {
    focused_view_idx: usize,
    buffer_views: Vec<BufferView>,
}

impl OpenedApplication {
    // TEMP:
    pub fn debug(&self, msg: String) -> Result<()> {
        execute!(io::stdout(), Print(msg))?;
        Ok(())
    }

    pub fn select_next_view(&mut self) -> Result<()> {
        if self.focused_view_idx != self.buffer_views.len() - 1 {
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
            UpdateView::Remove(file_path) => {
                self.buffer_views.remove(self.get_view_index(file_path));
                if self.buffer_views.is_empty() {
                    return Ok(Some(Command::Close));
                } else if self.focused_view_idx == self.buffer_views.len() {
                    self.focused_view_idx = self.focused_view_idx.saturating_sub(1);
                }
            }
            UpdateView::Reload(absolute_file_path) => {
                self.focused_view_idx = self.get_view_index(absolute_file_path)
            }
        }
        self.render_view()?;
        Ok(None)
    }

    /// Panics if path does not exist in buffer views
    fn get_view_index(&self, absolute_file_path: PathBuf) -> usize {
        self.buffer_views
            .iter()
            .position(|buffer_view| absolute_file_path == buffer_view.absolute_file_path)
            .expect("File path with update exists in application tabs.")
    }

    fn render_view(&self) -> Result<()> {
        execute!(io::stdout(), Clear(crossterm::terminal::ClearType::All))?;
        self.render_tabline()?;
        execute!(
            io::stdout(),
            Print(fs::read_to_string(
                self.buffer_views[self.focused_view_idx]
                    .absolute_file_path
                    .clone()
            )?)
        )?;
        Ok(())
    }

    fn render_tabline(&self) -> Result<()> {
        let mut stdout = io::stdout();
        stdout.execute(MoveTo(0, 0))?;

        for buffer_view in &self.buffer_views {
            let absolute_file_path = &buffer_view.absolute_file_path;
            let tab = format!(
                " {} ",
                absolute_file_path
                    .file_name()
                    .expect("Path to be a valid file name")
                    .to_string_lossy()
            )
            .with(
                if *absolute_file_path
                    == self.buffer_views[self.focused_view_idx].absolute_file_path
                {
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

#[derive(Debug)]
pub struct BufferView {
    absolute_file_path: PathBuf,
}

pub enum UpdateView {
    Remove(PathBuf),
    Reload(PathBuf),
}
