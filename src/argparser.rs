use std::{
    path::PathBuf,
    process::exit,
    time::{SystemTime, UNIX_EPOCH},
};

use clap::Parser;
use image::ImageFormat;

fn gen_filename(format: &ImageFormat) -> PathBuf {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Error with system clock!")
        .as_secs()
        .to_string();
    let f = format.extensions_str().concat();
    PathBuf::from(format!("Screenshot_{}", secs)).with_extension(f)
}

fn parse_path(s: &str) -> Result<PathBuf, String> {
    let mut path = PathBuf::from(s);
    if s.is_empty() {
        if let Ok(rshot_env_dir) = std::env::var("RSHOT_DIR") {
            path = PathBuf::from(rshot_env_dir);
        } else if let Some(home_dir) = std::env::home_dir() {
            path = home_dir.join("Pictures/Screenshots");

            if !path.exists() {
                std::fs::create_dir_all(&path).unwrap_or_else(|_| {
                    eprintln!("Could not create default directory: {:?}", path.to_str());
                    exit(1);
                });
            }
        }
    }

    if let Some(ext) = path.extension() {
        let _ = ImageFormat::from_extension(ext).unwrap_or_else(|| {
            eprintln!("Error: Unknown format: '{:?}'. Exiting...", ext);
            exit(1);
        });
    } else {
        path = path.join(gen_filename(&ImageFormat::Png));
    }

    Ok(path)
}

// impl ValueEnum for ImageFormat {
//     fn value_variants<'a>() -> &'a [Self] {}
//
//     fn from_str(input: &str, ignore_case: bool) -> Result<Self, String> {}
//     fn to_possible_value(&self) -> Option<clap::builder::PossibleValue> {}
// }

#[derive(Parser, Debug)]
#[command(version)]
pub struct Args {
    #[arg(value_parser = parse_path, default_missing_value = "")]
    output: Option<PathBuf>,
    // #[arg(long, short)]
    // format: Option<ImageFormat>,
    //
    #[arg(short, long)]
    pub no_prompt: bool,
}

impl Args {
    pub fn get_output_dir(&mut self) -> PathBuf {
        if self.output.is_none() {
            self.output = Some(parse_path("").unwrap());
        }

        self.output.clone().unwrap()
    }
}
