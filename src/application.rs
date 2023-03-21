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
            buffer_views: file_paths
                .into_iter()
                .map(|file_path| BufferView {
                    file_path,
                    offset: 0,
                })
                .collect(),
        };
        application.render_view()?;
        Ok(application)
    }
}

pub struct OpenedApplication {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    focused_view_idx: usize,
    buffer_views: Vec<BufferView>,
}

impl OpenedApplication {
    pub fn close(self) -> io::Result<ClosedApplication> {
        disable_raw_mode()?;
        execute!(io::stdout(), LeaveAlternateScreen, Show)?;
        Ok(ClosedApplication {})
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

    pub fn scroll_current_buffer(&mut self, steps: i16) -> Result<()> {
        self.buffer_views[self.focused_view_idx].scroll(steps);
        self.render_view()?;
        Ok(())
    }

    pub fn update_view(&mut self, update: UpdateView) -> Result<Option<Command>> {
        match update {
            UpdateView::Remove(file_paths) => {
                for removed_path in file_paths {
                    self.buffer_views.remove(self.get_view_index(removed_path));
                    if self.buffer_views.is_empty() {
                        return Ok(Some(Command::Close));
                    } else if self.focused_view_idx == self.buffer_views.len() {
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
        self.buffer_views
            .iter()
            .position(|buffer_view| file_path == *buffer_view.file_path)
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
        let mut tabs: Vec<Span> = Vec::with_capacity(self.buffer_views.len());
        for BufferView { file_path, .. } in &self.buffer_views {
            let absolute_file_path = file_path;
            let tab_name = format!(
                " {} ",
                absolute_file_path
                    .file_name()
                    .expect("Path to be a valid file name")
                    .to_string_lossy()
            );
            if *absolute_file_path == self.buffer_views[self.focused_view_idx].file_path {
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

        let curren_view = &self.buffer_views[self.focused_view_idx];
        // Skip if file can't be read, happens in rare cases when OS file
        // removals haven't had time to propagate through the file_watcher.
        if curren_view.file_path.exists() {
            let file_string = fs::read_to_string(curren_view.file_path.clone())?;
            Ok(Some(
                Paragraph::new(Text::raw(file_string)).scroll((curren_view.offset, 0)),
            ))
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

pub struct BufferView {
    file_path: PathBuf,
    offset: u16,
}

impl BufferView {
    pub fn scroll(&mut self, steps: i16) {
        self.offset = self.offset.saturating_add_signed(steps);
    }
}
