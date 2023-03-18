#[cfg(test)]
mod tests;

use pulldown_cmark::{Event, Parser, Tag};
use tui::{
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
};

fn format(markdown_str: &str) -> Text {
    Text::from(markdown_to_spans(markdown_str))
}

fn markdown_to_spans(text: &str) -> Vec<Spans> {
    let parser = Parser::new(text);
    let mut lines: Vec<Spans> = Vec::new();
    let mut styles: Vec<Style> = Vec::new();
    let mut current_line: Vec<Span> = Vec::new();
    for event in parser {
        match event {
            Event::Start(tag) => {
                let mut style = Style::default();
                match tag {
                    Tag::Strong => style = style.add_modifier(Modifier::BOLD),
                    Tag::Emphasis => style = style.add_modifier(Modifier::ITALIC),
                    Tag::Paragraph => {
                        if !current_line.is_empty() {
                            lines.push(Spans::from(current_line));
                            current_line = Vec::new();
                        }
                    }
                    _ => style = style.bg(Color::Reset),
                };
                styles.push(style);
            }
            Event::HardBreak => {
                lines.push(Spans::from(current_line));
                current_line = Vec::new();
            }
            Event::SoftBreak => {
                lines.push(Spans::from(current_line));
                current_line = Vec::new();
            }
            Event::Text(str) => {
                let span = Span::styled(str, styles.pop().unwrap());
                current_line.push(span);
            }
            _ => continue,
        }
    }
    lines.push(Spans::from(current_line));
    lines
}
