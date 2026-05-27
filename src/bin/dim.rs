use anyhow::Result;
use clap::Parser;
use dim::*;
use humantime::Duration;
use std::fs::File;
use std::path::PathBuf;
use std::process::exit;

#[derive(Debug, Parser)]
/// Dim smoothly controls screen brightness
///
/// ## Examples
///
/// Show the current brightness
///
/// `dim`
///
/// Increase the brightness by 10%
///
/// `dim -- +10%`
///
/// // Decrease the brightness by 10%
///
/// `dim -- -10%`
///
/// Dim the screen to 30% brightness over 3 seconds, storing the current brightness in the
/// statefile:
///
/// `dim --save --duration 3s 30%`
///
/// Restore the screen to the previously saved brightness, using 2 seconds:
///
/// `dim --restore --duration 2s`
struct Args {
    /// Path to the file to write to set the brightness. We'll try to pick this from
    /// `/sys/class/backlight` if not set.
    ///
    #[arg(
        long = "set-brightness-path",
        hide_short_help = true,
        hide_long_help = true
    )]
    brightness_file: Option<PathBuf>,

    /// Path to the file to read the current brightness from. This can be the same file as the file to
    /// set the brightness.  We'll try to pick this from `/sys/class/backlight` if not set.
    ///
    #[arg(
        long = "get-brightness-path",
        hide_short_help = true,
        hide_long_help = true
    )]
    current_brightness_file: Option<PathBuf>,

    /// Path to the file to read the maximum possible brightness from. We'll try to pick this
    /// from `/sys/class/backlight` if not set.
    ///
    #[arg(
        long = "max-brightness-path",
        hide_short_help = true,
        hide_long_help = true
    )]
    max_brightness_file: Option<PathBuf>,

    /// The state file is used to keep track of the original brightness, so we
    /// can later restore it.
    ///
    #[arg(long, hide_short_help = true)]
    state_file: Option<PathBuf>,

    /// How long it should take for the screen to go from it's current
    /// brightness to zero brightness.
    ///
    #[arg(long, default_value = "1s")]
    duration: Duration,

    /// How many times per second the brightness will be updated.
    ///
    #[arg(long, default_value = "60")]
    framerate: u64,

    /// Save the current brightness to the statefile.
    ///
    #[arg(long, short)]
    save: bool,

    /// Restore previously saved brightness from the statefile.
    ///
    #[arg(long, short)]
    restore: bool,

    /// The brightness to target. Can either be an absolute value between 0 and the value in the
    /// file at `max-brightness-path`, or an percentage (e.g. "0%" to "100%").
    ///
    #[arg()]
    target_str: Option<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let brightness_file = args.brightness_file.unwrap_or(find_file("brightness")?);

    let current_brightness_file = args
        .current_brightness_file
        .unwrap_or(find_file("actual_brightness")?);

    let max_brightness_file = args
        .max_brightness_file
        .unwrap_or(find_file("max_brightness")?);

    let state_file = args.state_file.unwrap_or_else(|| {
        let dirs = xdg::BaseDirectories::with_prefix("dim");
        dirs.place_config_file("stored_brightness")
            .expect("Failed to create xdg config path")
    });

    let duration = args.duration.as_secs();

    let current: Brightness = Brightness::from_file(&current_brightness_file)?;
    let maximum: Brightness = Brightness::from_file(&max_brightness_file)?;

    if args.save {
        save(&state_file, current)?;
    }

    let target: Brightness = if args.restore {
        Brightness::from_file(state_file)?
    } else if let Some(target_str) = args.target_str {
        Brightness::parse(&target_str, current, maximum)?
    } else {
        let pct = (current / maximum) * 100.0;
        println!("Current brightness: {current} / {maximum} ({:.2}%)", pct);
        exit(0);
    };
    let target = if target > maximum { maximum } else { target };

    let total_frames = duration * args.framerate;

    let (step_size, dimming): (u64, bool) = match (*target, *current) {
        (t, o) if t > o => ((t - o) / total_frames, false),
        (t, o) if o > t => ((o - t) / total_frames, true),
        (_t, _o) => exit(0),
    };

    let output = File::create(&brightness_file)?;
    let mut brightness = current;
    for _i in 0..total_frames {
        if dimming {
            if *brightness < step_size {
                brightness = Brightness::new(0);
            } else {
                brightness = brightness - step_size;
            }
        } else if *(target - *brightness) < step_size {
            brightness = target;
        } else {
            brightness = brightness + step_size;
        }

        set_brightness(&output, brightness)?;
        std::thread::sleep(std::time::Duration::from_millis(1000 / 60));
    }
    Ok(())
}
