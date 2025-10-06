use std::{
    path::PathBuf,
    process::exit,
    str::FromStr,
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

// impl FromStr for PathBuf {}

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
//

#[derive(Debug, Default, Clone)]
pub struct RegionSelect {
    pub top_left_origin: Point,
    pub size: Point,
}

#[derive(Debug, Default, Clone)]
pub struct Point {
    pub x: u32,
    pub y: u32,
}

impl FromStr for RegionSelect {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts = s.split_whitespace().collect::<Vec<&str>>();
        if parts.is_empty() || parts.len() > 2 {
            return Err("Correct region format is '[X,Y] WxH'".to_string());
        }

        let mut origin = Point::default();
        let mut size_idx = 0;
        if parts.len() == 2 {
            let origin_parts: Vec<&str> = parts[0].split(',').collect();
            if origin_parts.len() != 2 {
                return Err("Correct region format is '[X,Y] WxH'".to_string());
            }

            let x = origin_parts[0]
                .parse::<u32>()
                .map_err(|_| "Region format requires only integers!".to_string())?;

            let y = origin_parts[1]
                .parse::<u32>()
                .map_err(|_| "Region format requires only integers!".to_string())?;

            origin = Point { x, y };
            size_idx = 1;
        }

        let size_parts: Vec<&str> = parts[size_idx].split('x').collect();
        if size_parts.len() != 2 {
            return Err("Correct region format is '[X,Y] WxH'".to_string());
        }

        let w = size_parts[0]
            .parse::<u32>()
            .map_err(|_| "Region format requires only integers!".to_string())?;

        let h = size_parts[1]
            .parse::<u32>()
            .map_err(|_| "Region format requires only integers!".to_string())?;

        Ok(RegionSelect {
            top_left_origin: origin,
            size: Point { x: w, y: h },
        })
    }
}

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

    #[arg(short, long)]
    pub region: Option<RegionSelect>,

    #[arg(long)]
    pub dry_run: bool,
}

impl Args {
    pub fn get_output_dir(&mut self) -> PathBuf {
        if self.output.is_none() {
            self.output = Some(parse_path("").unwrap());
        }

        self.output.clone().unwrap()
    }
}
