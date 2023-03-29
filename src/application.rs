use anyhow::Result;
use crossterm::{
    cursor::{Hide, Show},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction},
    style::{Color, Style},
    text::{Span, Spans, Text},
    widgets::{Paragraph, Widget, Wrap},
    Terminal,
};
use std::{
    fs,
    io::{self, Stdout},
    path::PathBuf,
};
use strum::{EnumIter, IntoEnumIterator};

use crate::{markdown_renderer::render_markdown, Command};

pub struct ClosedApplication;
impl ClosedApplication {
    pub fn open(file_paths: Vec<PathBuf>) -> Result<OpenedApplication> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, Hide)?;
        let mut application = OpenedApplication {
            terminal: Terminal::new_split(
                CrosstermBackend::new(stdout),
                vec![Constraint::Length(1), Constraint::Min(1)],
                Direction::Vertical,
            )?,
            focused_view_idx: 0,
            buffer_views: file_paths
                .into_iter()
                .map(|file_path| MarkdownView {
                    file_path,
                    offset: 0,
                })
                .collect(),
        };
        application.draw_all()?;
        Ok(application)
    }
}

pub struct OpenedApplication {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    focused_view_idx: usize,
    buffer_views: Vec<MarkdownView>,
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
        self.draw_all()?;
        Ok(())
    }

    pub fn select_prev_view(&mut self) -> Result<()> {
        self.focused_view_idx = self.focused_view_idx.saturating_sub(1);
        self.draw_all()?;
        Ok(())
    }

    // TODO: fill viewportscroll
    pub fn scroll_current_buffer(&mut self, steps: i16) -> Result<()> {
        self.buffer_views[self.focused_view_idx].scroll(steps);
        self.draw_viewport(ViewportIndex::Markdown)?;
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
                self.draw_all()?
            }
            UpdateView::Reload(file_paths) => {
                for updated_path in file_paths {
                    self.focused_view_idx = self.get_view_index(updated_path)
                }
                self.draw_viewport(ViewportIndex::Markdown)?;
            }
            // Just re-render the view
            UpdateView::Resize => self.draw_all()?,
        }
        Ok(None)
    }

    fn get_view_index(&self, file_path: PathBuf) -> usize {
        self.buffer_views
            .iter()
            .position(|buffer_view| file_path == *buffer_view.file_path)
            .expect("File path with update exists in application tabs.")
    }

    fn draw_all(&mut self) -> Result<()> {
        for viewport_index in ViewportIndex::iter() {
            self.draw_viewport(viewport_index)?;
        }
        Ok(())
    }

    fn draw_viewport(&mut self, viewport_index: ViewportIndex) -> Result<()> {
        self.terminal.clear_viewport(viewport_index as usize);
        match viewport_index {
            ViewportIndex::Tabline => {
                let tab_widget = self.tabline_widget();
                self.terminal
                    .render_widget_on_viewport(tab_widget, viewport_index as usize);
            }
            ViewportIndex::Markdown => {
                let focused_buffer = &self.buffer_views[self.focused_view_idx];
                let found_buffer_view_widget = Self::markdown_view_widget(focused_buffer)?;
                if let Some(buffer_view_widget) = found_buffer_view_widget {
                    self.terminal
                        .render_widget_on_viewport(buffer_view_widget, viewport_index as usize);
                }
            }
        }
        self.terminal
            .flush_viewport_region(viewport_index as usize)?;
        Ok(())
    }

    fn tabline_widget(&mut self) -> impl Widget {
        let mut tabs: Vec<Span> = Vec::with_capacity(self.buffer_views.len());
        for MarkdownView { file_path, .. } in &self.buffer_views {
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

    fn markdown_view_widget(markdown_view: &MarkdownView) -> Result<Option<Paragraph>> {
        // Skip if file can't be read, happens in rare cases when OS file
        // removals haven't had time to propagate through the file_watcher.
        if markdown_view.file_path.exists() {
            let file_string = fs::read_to_string(markdown_view.file_path.clone())?;
            // TODO: add render here:
            let rendered_markdown = render_markdown(file_string);
            Ok(Some(
                Paragraph::new(rendered_markdown)
                    .scroll((markdown_view.offset, 0))
                    .wrap(Wrap { trim: true }),
            ))
        } else {
            Ok(None)
        }
    }
}

#[derive(EnumIter, Copy, Clone)]
enum ViewportIndex {
    Tabline = 0,
    Markdown = 1,
}

pub enum UpdateView {
    Remove(Vec<PathBuf>),
    Reload(Vec<PathBuf>),
    Resize,
}

struct MarkdownView {
    file_path: PathBuf,
    offset: u16,
}

impl MarkdownView {
    pub fn scroll(&mut self, steps: i16) {
        self.offset = self.offset.saturating_add_signed(steps);
    }
}
