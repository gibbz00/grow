use anyhow::Result;
use crossterm::{
    cursor::{Hide, Show},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{
    fs,
    io::{self, Stdout},
    path::PathBuf,
};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Span, Spans, Text},
    widgets::{Paragraph, Widget},
    Frame, Terminal,
};

use crate::Command;

pub struct ClosedApplication;
impl ClosedApplication {
    pub fn open(file_paths: Vec<PathBuf>) -> Result<OpenedApplication> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, Hide)?;
        let mut application = OpenedApplication {
            terminal: Terminal::new(CrosstermBackend::new(stdout))?,
            focused_view_idx: 0,
            file_paths,
        };
        application.render_view()?;
        Ok(application)
    }
}

pub struct OpenedApplication {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    focused_view_idx: usize,
    file_paths: Vec<PathBuf>,
}

impl OpenedApplication {
    pub fn close(self) -> io::Result<ClosedApplication> {
        disable_raw_mode()?;
        execute!(io::stdout(), LeaveAlternateScreen, Show)?;
        Ok(ClosedApplication {})
    }

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

    fn get_view_index(&self, file_path: PathBuf) -> usize {
        self.file_paths
            .iter()
            .position(|self_paths| file_path == *self_paths)
            .expect("File path with update exists in application tabs.")
    }

    fn render_view(&mut self) -> Result<()> {
        let tab_widget = self.tabline_widget();
        let found_buffer_view_widget = self.buffer_view_widget()?;
        self.terminal.draw(|frame| {
            let widget_sizes = Self::get_ui_widget_sizes(frame);
            frame.render_widget(tab_widget, widget_sizes.tabline);
            if let Some(buffer_view_widget) = found_buffer_view_widget {
                frame.render_widget(buffer_view_widget, widget_sizes.buffer_view);
            }
        })?;
        Ok(())
    }

    fn tabline_widget(&mut self) -> impl Widget {
        let mut tabs: Vec<Span> = Vec::with_capacity(self.file_paths.len());
        for file_path in &self.file_paths {
            let absolute_file_path = file_path;
            let tab_name = format!(
                " {} ",
                absolute_file_path
                    .file_name()
                    .expect("Path to be a valid file name")
                    .to_string_lossy()
            );
            if *absolute_file_path == self.file_paths[self.focused_view_idx] {
                tabs.push(Span::raw(tab_name));
            } else {
                tabs.push(Span::styled(tab_name, Style::default().fg(Color::DarkGray)))
            }
        }
        let tabline = Paragraph::new(Text::from(Spans::from(tabs)));
        tabline
    }

    fn buffer_view_widget(&mut self) -> Result<Option<impl Widget>> {
        // let markdown_string = fs::read_to_string(Path::new("README.md")).unwrap();
        // execute!(stdout, Print(markdown_string))?;

        let current_file_path = &self.file_paths[self.focused_view_idx];
        // Skip if file can't be read, happens in rare cases when OS file
        // removals haven't had time to propagate through the file_watcher.
        if current_file_path.exists() {
            let file_string = fs::read_to_string(current_file_path.clone())?;
            Ok(Some(Paragraph::new(Text::raw(file_string))))
        } else {
            Ok(None)
        }
    }

    fn get_ui_widget_sizes<B: Backend>(frame: &mut Frame<B>) -> WidgetSizes {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Min(1)])
            .split(frame.size());
        WidgetSizes {
            tabline: layout[0],
            buffer_view: layout[1],
        }
    }
}

pub struct WidgetSizes {
    tabline: Rect,
    buffer_view: Rect,
}

pub enum UpdateView {
    Remove(Vec<PathBuf>),
    Reload(Vec<PathBuf>),
}
