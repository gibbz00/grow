use anyhow::{anyhow, Result};
use clap::Parser;
use std::{fs, path::PathBuf};

#[derive(Parser)]
#[command(author, version, about)]
pub struct Args {
    /// Markdown file to render view from
    #[arg(required = true)]
    pub files: Vec<PathBuf>,
}

pub fn parse() -> Result<Args> {
    let args = Args::parse();
    let mut good_file_paths: Vec<PathBuf> = Vec::new();
    let mut bad_file_paths: Vec<PathBuf> = Vec::new();
    for file_path in args.files {
        match fs::canonicalize(file_path.clone()) {
            Ok(absolute_file_path) => {
                good_file_paths.push(absolute_file_path);
            }
            Err(_) => {
                bad_file_paths.push(file_path);
            }
        }
    }
    if !bad_file_paths.is_empty() {
        Err(anyhow!("Failed to locate files: {:?}", bad_file_paths))
    } else {
        Ok(Args {
            files: good_file_paths,
        })
    }
}
