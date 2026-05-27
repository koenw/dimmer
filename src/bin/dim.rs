use anyhow::Result;
use clap::Parser;
use dim::*;
use humantime::Duration;
use std::fs::File;
use std::path::PathBuf;
use std::process::exit;

#[cfg(feature = "dbus")]
const DEFAULT_DIMMER: &str = "dbus";
#[cfg(not(feature = "dbus"))]
const DEFAULT_DIMMER: &str = "sysfs";

#[derive(clap::ValueEnum, Clone, Debug)]
enum Dimmer {
    Sysfs,
    Dbus,
}

#[derive(Debug, Parser)]
/// Smoothly control your screen's brightness
///
/// ## Examples
///
/// Show the current brightness
///
/// `dim`
///
/// Increase the brightness by 10%
///
/// `dim +10%`
///
/// Decrease the brightness by 10%
///
/// `dim -10%`
///
/// Save the current brightness
///
/// dim --save
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
    #[arg(
        long = "set-brightness-path",
        hide_short_help = true,
        hide_long_help = true
    )]
    brightness_file: Option<PathBuf>,

    /// How to change the brightness
    #[arg(long, value_enum, default_value = DEFAULT_DIMMER)]
    dimmer: Dimmer,

    /// The state file is used to save the current brightness, so we can later restore it.
    #[arg(long, hide_short_help = true)]
    state_file: Option<PathBuf>,

    /// How long it should take for the screen to go from it's current
    /// brightness to zero brightness.
    #[arg(long, default_value = "330ms")]
    duration: Duration,

    /// How many times per second the brightness will be updated.
    #[arg(long, default_value = "60")]
    framerate: u32,

    /// Save the current brightness to the statefile.
    #[arg(long, short)]
    save: bool,

    /// Restore previously saved brightness from the statefile.
    #[arg(long, short)]
    restore: bool,

    /// The brightness to target. Can either be an absolute value between 0 and the value in the
    /// file at `max-brightness-path`, or an percentage (e.g. "0%" to "100%").
    #[arg(allow_hyphen_values = true)]
    target_str: Option<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let brightness_file = args
        .brightness_file
        .unwrap_or(Brightness::find_file("brightness")?);

    let state_file = args.state_file.unwrap_or_else(|| {
        let dirs = xdg::BaseDirectories::with_prefix("dim");
        dirs.place_config_file("stored_brightness")
            .expect("Failed to create xdg config path")
    });

    let dimmer: &mut dyn dim::Dimmer = match args.dimmer {
        Dimmer::Sysfs => {
            let f = File::create(&brightness_file)?;
            &mut dim::dimmer::Sysfs::with_file(f)
        }
        Dimmer::Dbus => &mut dim::dimmer::Dbus::new()?,
    };

    let duration: f32 = args.duration.as_secs_f32();

    let current = Brightness::current()?;
    let maximum = Brightness::max()?;

    if args.save {
        current.save(&state_file)?;
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

    let total_frames = (duration * args.framerate as f32).floor() as u32;

    let (step_size, dimming): (u32, bool) = match (*target, *current) {
        (t, o) if t > o => ((t - o) / total_frames, false),
        (t, o) if o > t => ((o - t) / total_frames, true),
        (_t, _o) => exit(0),
    };

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

        dimmer.set(brightness)?;
        std::thread::sleep(std::time::Duration::from_millis(1000 / 60));
    }
    Ok(())
}
