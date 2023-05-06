use anyhow::{Context, Result};
use glob::glob;
use humantime::Duration;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::exit;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
/// Dimmer smoothly transitions your screen from one brightness to another.
struct Opt {
    /// Path to the file to write to set the brightness. We'll try to pick this from
    /// `/sys/class/backlight` if not set.
    #[structopt(long="set-brightness-path", parse(from_os_str))]
    brightness_file: Option<PathBuf>,

    /// Path to the file to read the current brightness from. This can be the same file as the file to
    /// set the brightness.  We'll try to pick this from `/sys/class/backlight` if not set.
    #[structopt(long="get-brightness-path", parse(from_os_str))]
    current_brightness_file: Option<PathBuf>,

    /// Path to the file to read the maximum possible brightness from. We'll try to pick this
    /// from `/sys/class/backlight` if not set.
    #[structopt(long="max-brightness-path", parse(from_os_str))]
    max_brightness_file: Option<PathBuf>,

    /// The state file is used to keep track of the original brightness, so we
    /// can later restore it.
    #[structopt(long, parse(from_os_str))]
    state_file: Option<PathBuf>,

    /// How long it should take for the screen to go from it's current
    /// brightness to zero brightness.
    #[structopt(long, default_value = "5s")]
    duration: Duration,

    /// The brightness to target.
    #[structopt(long, default_value = "0")]
    target: u64,

    /// How many times per second the brightness will be updated.
    #[structopt(long, default_value = "60")]
    framerate: u64,

    /// Save the current brightness to the statefile.
    #[structopt(long="save", short)]
    save_brightness: bool,

    /// Restore previously saved brightness from the statefile.
    #[structopt(long="restore", short)]
    restore_brightness: bool,
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

    let original_brightness = get_brightness(&current_brightness_file)?;
    let max_brightness = get_brightness(&max_brightness_file)?;

    if opt.save_brightness {
        save_brightness(&state_file, original_brightness)?;
    }

    let target_brightness = if opt.restore_brightness {
        get_brightness(state_file)?
    } else {
        opt.target
    };
    let target_brightness = if target_brightness > max_brightness { max_brightness } else { target_brightness };

    let total_frames = duration * opt.framerate;

    let (step_size, dimming) = match (target_brightness, original_brightness) {
        (t, o) if t > o => ((t - o) / total_frames, false),
        (t, o) if o > t => ((o - t) / total_frames, true),
        (_t, _o) => exit(0),
    };

    let output = File::create(&brightness_file)?;
    let mut brightness = original_brightness;
    for _i in 0..total_frames {
        if dimming {
            if brightness < step_size {
                brightness = 0;
            } else {
                brightness -= step_size;
            }
        } else if (target_brightness - brightness) < step_size {
            brightness = target_brightness;
        } else {
            brightness += step_size;
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

fn get_brightness<P: AsRef<Path>>(brightness_file: P) -> Result<u64> {
    let path = brightness_file.as_ref();
    let res = std::fs::read_to_string(path)
        .context("Failed to read {brightness_file}")?
        .trim()
        .parse()
        .context("Failed to parse brightness from {brightness_file}")?;
    Ok(res)
}

fn set_brightness<F: Write>(mut f: F, brightness: u64) -> Result<()> {
    write!(f, "{brightness}")?;
    Ok(())
}

fn save_brightness<P: AsRef<Path>>(state_file: P, brightness: u64) -> Result<()> {
    let mut output = File::create(&state_file)?;
    write!(output, "{brightness}")?;
    Ok(())
}
