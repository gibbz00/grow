#[cfg(test)]
mod tests;

use pulldown_cmark::{Event, Parser, Tag};
use ratatui::{
    style::{Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{List, ListItem, Paragraph, Widget, Wrap},
};

pub fn parse_markdown_to_widgets(markdown_str: String) -> Vec<Box<dyn Widget>> {
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
    // TODO: continually push widgets when parsing
    // TEMP: list just to test multiple widget functionality
    vec![
        Box::new(Paragraph::new(Text::from(lines)).wrap(Wrap { trim: true })),
        Box::new(List::new([
            ListItem::new("Item 1"),
            ListItem::new("Item 2"),
            ListItem::new("Item 3"),
        ])),
    ]
}
