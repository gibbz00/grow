use crate::markdown_renderer::render_markdown;
use ratatui::{
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
fn nested_styles() {
    compare_style(
        "***",
        Style::default()
            .add_modifier(Modifier::ITALIC)
            .add_modifier(Modifier::BOLD),
    )
}

fn compare_style(enclosing_str: &str, style: Style) {
    let inner_str = "Test";
    let markdown_string = format!("{enclosing_str}{inner_str}{enclosing_str}");
    assert_eq!(
        Text::styled(inner_str, style),
        render_markdown(markdown_string)
    )
}

// #[test]
// fn compare_ansi() {
//     let test_ansi = b"n";
//     let result_buffer = BufWriter::new(std::io::stdout());
//     // execute!(result_buffer, )
//     assert_eq!(result_buffer.buffer(), test_ansi)
// }
