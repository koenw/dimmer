use anyhow::{Context, Result};
use glob::glob;
use humantime::Duration;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::exit;
use structopt::StructOpt;
use thiserror::Error;

#[derive(Error, Debug)]
enum DimmerError {
    #[error("Invalid percentage given by user")]
    InvalidPercentage,
    #[error("Failed to parse invalid Brightness")]
    InvalidBrightness(#[from] std::num::ParseIntError),
}

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
struct Brightness(u64);

impl std::fmt::Display for Brightness {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::str::FromStr for Brightness {
    type Err = DimmerError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        Ok(input.parse::<u64>().map(Brightness)?)
    }
}

impl Brightness {
    fn parse_with_percentage(input: &str, max: Brightness) -> Result<Brightness> {
        match input.strip_suffix('%') {
            Some(percentage) => {
                let percentage = percentage.parse::<u64>()?;
                if percentage > 100 {
                    return Err(DimmerError::InvalidPercentage.into());
                }
                Ok(Brightness(
                    ((percentage as f64 / 100.0) * max.0 as f64) as u64,
                ))
            }
            None => Ok(input.parse::<u64>().map(Brightness)?),
        }
    }

    fn from_file<P: AsRef<Path>>(path: P) -> Result<Brightness> {
        let path = path.as_ref();
        let res = std::fs::read_to_string(path)
            .context("Failed to read {path}")?
            .trim()
            .parse()
            .context("Failed to parse brightness from {path}")?;
        Ok(res)
    }
}

#[derive(Debug, StructOpt)]
/// Dimmer smoothly transitions your screen from one brightness to another.
struct Opt {
    /// Path to the file to write to set the brightness. We'll try to pick this from
    /// `/sys/class/backlight` if not set.
    #[structopt(long = "set-brightness-path", parse(from_os_str))]
    brightness_file: Option<PathBuf>,

    /// Path to the file to read the current brightness from. This can be the same file as the file to
    /// set the brightness.  We'll try to pick this from `/sys/class/backlight` if not set.
    #[structopt(long = "get-brightness-path", parse(from_os_str))]
    current_brightness_file: Option<PathBuf>,

    /// Path to the file to read the maximum possible brightness from. We'll try to pick this
    /// from `/sys/class/backlight` if not set.
    #[structopt(long = "max-brightness-path", parse(from_os_str))]
    max_brightness_file: Option<PathBuf>,

    /// The state file is used to keep track of the original brightness, so we
    /// can later restore it.
    #[structopt(long, parse(from_os_str))]
    state_file: Option<PathBuf>,

    /// How long it should take for the screen to go from it's current
    /// brightness to zero brightness.
    #[structopt(long, default_value = "5s")]
    duration: Duration,

    /// The brightness to target. Can either be an absolute value between 0 and the value in the
    /// file at `max-brightness-path`, or an percentage (e.g. "0%" to "100%").
    #[structopt(long = "target", default_value = "0")]
    target_str: String,

    /// How many times per second the brightness will be updated.
    #[structopt(long, default_value = "60")]
    framerate: u64,

    /// Save the current brightness to the statefile.
    #[structopt(long, short)]
    save: bool,

    /// Restore previously saved brightness from the statefile.
    #[structopt(long, short)]
    restore: bool,
}

const SYS_BACKLIGHT_PREFIX: &str = "/sys/class/backlight";

fn main() -> Result<()> {
    let opt = Opt::from_args();

    let brightness_file = opt.brightness_file.unwrap_or(find_file("brightness")?);

    let current_brightness_file = opt
        .current_brightness_file
        .unwrap_or(find_file("actual_brightness")?);

    let max_brightness_file = opt
        .max_brightness_file
        .unwrap_or(find_file("max_brightness")?);

    let state_file = opt.state_file.unwrap_or_else(|| {
        let dirs = xdg::BaseDirectories::with_prefix("dimmer")
            .expect("Failed to setup XDG base directories");
        dirs.place_config_file("stored_brightness")
            .expect("Failed to create xdg config path")
    });

    let duration = opt.duration.as_secs();

    let stored: Brightness = Brightness::from_file(&current_brightness_file)?;
    let maximum: Brightness = Brightness::from_file(&max_brightness_file)?;

    if opt.save {
        save(&state_file, stored)?;
    }

    let target: Brightness = if opt.restore {
        Brightness::from_file(state_file)?
    } else {
        Brightness::parse_with_percentage(&opt.target_str, maximum)?
    };
    let target = if target > maximum { maximum } else { target };

    let total_frames = duration * opt.framerate;

    let (step_size, dimming): (u64, bool) = match (target.0, stored.0) {
        (t, o) if t > o => ((t - o) / total_frames, false),
        (t, o) if o > t => ((o - t) / total_frames, true),
        (_t, _o) => exit(0),
    };

    let output = File::create(&brightness_file)?;
    let mut brightness = stored;
    for _i in 0..total_frames {
        if dimming {
            if brightness.0 < step_size {
                brightness = Brightness(0);
            } else {
                brightness = Brightness(brightness.0 - step_size);
            }
        } else if (target.0 - brightness.0) < step_size {
            brightness = target;
        } else {
            brightness = Brightness(brightness.0 + step_size);
        }

        set_brightness(&output, brightness)?;
        std::thread::sleep(std::time::Duration::from_millis(1000 / 60));
    }
    Ok(())
}

fn find_file(filename: &str) -> Result<PathBuf> {
    let glob_path = format!("{SYS_BACKLIGHT_PREFIX}/*/{filename}");
    let path = glob(&glob_path)
        .context("Failed to glob {glob_path}")?
        .next()
        .context("Failed to find match at {glob_path}")?
        .context("Glob error trying to match {glob_path}")?;
    Ok(path)
}

fn set_brightness<F: Write>(mut f: F, brightness: Brightness) -> Result<()> {
    write!(f, "{}", brightness.0)?;
    Ok(())
}

fn save<P: AsRef<Path>>(state_file: P, brightness: Brightness) -> Result<()> {
    let mut output = File::create(&state_file)?;
    write!(output, "{}", brightness.0)?;
    Ok(())
}
