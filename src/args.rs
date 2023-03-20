use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about)]
pub struct Args {
    /// Markdown file to render view from
    #[arg(required = true)]
    pub files: Vec<PathBuf>,
}
