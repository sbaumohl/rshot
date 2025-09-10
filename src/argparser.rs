use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(version)]
pub struct Args {
    #[arg(short, long = "output", id = "output")]
    output_dir: Option<String>,
}

impl Args {
    pub fn get_output_dir(&self) -> PathBuf {
        if let Some(path) = self.output_dir.as_ref() {
            PathBuf::from(path)
        } else {
            let home_dir: PathBuf =
                std::env::home_dir().expect("Environment variable HOME not set!");
            home_dir.join("Pictures/Screenshots")
        }
    }
}
