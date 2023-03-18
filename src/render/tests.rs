use std::io::BufWriter;

use crossterm::execute;
use tui::{
    style::{Modifier, Style},
    text::{Span, Spans, Text},
};

#[test]
fn strong_is_bold() {
    compare_style("**", Style::default().add_modifier(Modifier::BOLD))
}

#[test]
fn emphasis_is_italics() {
    compare_style("*", Style::default().add_modifier(Modifier::ITALIC))
}

#[test]
fn line_contains_two_styles() {
    let markdown_str = "*italic* **bold**";
    let expected = Text::from(Spans::from(vec![
        Span::styled("italic", Style::default().add_modifier(Modifier::ITALIC)),
        Span::raw(" "),
        Span::styled("bold", Style::default().add_modifier(Modifier::BOLD)),
    ]));
    assert_eq!(expected, super::format(markdown_str))
}

#[test]
fn no_newline_before_first_paragraph() {}

#[test]
fn last_line_is_flushed() {}

fn compare_style(enclosing_str: &str, style: Style) {
    let inner_str = "Test";
    let markdown_string = format!("{enclosing_str}{inner_str}{enclosing_str}");
    assert_eq!(
        Text::styled(inner_str, style),
        super::format(&markdown_string)
    )
}

#[test]
fn compare_ansi() {
    let test_ansi = b"n";
    let result_buffer = BufWriter::new(std::io::stdout());
    // execute!(result_buffer, )
    assert_eq!(result_buffer.buffer(), test_ansi)
}
