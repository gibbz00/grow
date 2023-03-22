#[cfg(test)]
mod tests;

use pulldown_cmark::{Event, Parser, Tag};
use ratatui::{
    style::{Modifier, Style},
    text::{Span, Spans, Text},
};

pub fn render_markdown<'a>(markdown_str: String) -> Text<'a> {
    let parser = Parser::new(&markdown_str);
    let mut lines: Vec<Spans> = Vec::new();
    let mut curr_style: Style = Style::default();
    let mut current_line: Vec<Span> = Vec::new();
    for event in parser {
        match event {
            Event::Start(tag) => {
                // // TEMP: debuggin
                // current_line.push(Span::styled(
                //     format!("{tag:?}"),
                //     Style::default().bg(Color::Red),
                // ));
                match tag {
                    Tag::Strong => curr_style = curr_style.add_modifier(Modifier::BOLD),
                    Tag::Emphasis => curr_style = curr_style.add_modifier(Modifier::ITALIC),
                    _ => (),
                };
            }
            Event::End(tag) => {
                // // TEMP: debuggin
                // current_line.push(Span::styled(
                //     format!("{tag:?}"),
                //     Style::default().bg(Color::Red),
                // ));
                match tag {
                    Tag::Strong => curr_style = curr_style.remove_modifier(Modifier::BOLD),
                    Tag::Emphasis => curr_style = curr_style.remove_modifier(Modifier::ITALIC),
                    Tag::Paragraph => {
                        lines.push(Spans::from(current_line));
                        current_line = Vec::new();
                    }
                    _ => (),
                };
            }
            Event::Text(str) => {
                let span = Span::styled(str.into_string(), curr_style);
                current_line.push(span);
            }
            _ => continue,
        }
    }
    Text::from(lines)
}
