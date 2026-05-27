use crate::change::*;
use crate::DimError;
use glob::glob;
use std::io::Write;
use std::ops::{Add, Deref, Sub};
use std::path::Path;
use std::path::PathBuf;
use std::str::FromStr;

pub const SYS_BACKLIGHT_PREFIX: &str = "/sys/class/backlight";

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub struct Brightness(u64);

impl Deref for Brightness {
    type Target = u64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::fmt::Display for Brightness {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::str::FromStr for Brightness {
    type Err = DimError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        Ok(input.parse::<u64>().map(Brightness)?)
    }
}

impl Add<u64> for Brightness {
    type Output = Brightness;

    fn add(self, other: u64) -> Brightness {
        Brightness(self.0 + other)
    }
}

impl Sub<u64> for Brightness {
    type Output = Brightness;

    fn sub(self, other: u64) -> Brightness {
        Brightness(self.0 - other)
    }
}

impl std::ops::Div for Brightness {
    type Output = f64;

    fn div(self, other: Brightness) -> Self::Output {
        self.0 as f64 / other.0 as f64
    }
}

impl Brightness {
    pub fn new(value: u64) -> Brightness {
        Brightness(value)
    }

    pub fn current() -> Result<Self, DimError> {
        Self::from_file(Self::find_file("actual_brightness")?)
    }

    pub fn max() -> Result<Self, DimError> {
        Self::from_file(Self::find_file("max_brightness")?)
    }

    fn write_file(&self, f: impl AsRef<Path>) -> Result<(), DimError> {
        let mut f = std::fs::File::create(f.as_ref())?;
        write!(f, "{}", self)?;
        Ok(())
    }

    pub fn save(&self, state_file: impl AsRef<Path>) -> Result<(), DimError> {
        self.write_file(state_file)
    }

    #[cfg(not(feature = "dbus"))]
    pub fn set(&self, f: &impl Write) -> Result<Self, DimError> {
        //let f = std::fs::File::Create(Self::find_file("brightness")?)?;
        write!(f, "{}", self)?;
    }

    #[cfg(feature = "dbus")]
    pub fn set(&self, _f: &impl std::io::Write) -> Result<(), DimError> {
        let conn = dbus::blocking::Connection::new_system()?;
        let proxy = conn.with_proxy(
            "org.freedesktop.login1",
            "/org/freedesktop/login1/session/auto",
            std::time::Duration::from_millis(100),
        );
        let _: () = proxy.method_call(
            "org.freedesktop.login1.Session",
            "SetBrightness",
            ("backlight", "intel_backlight", self.0 as u32),
        )?;

        Ok(())
    }

    pub fn parse(
        input: &str,
        current: Brightness,
        max: Brightness,
    ) -> Result<Brightness, DimError> {
        let change = Change::from_str(input)?;

        match change {
            Change {
                direction: ChangeDirection::Increase,
                magnitude: Magnitude::Percentage(percentage),
            } => {
                let fraction = (current.0 as f64 / 100.0) * percentage;
                Ok(Brightness(std::cmp::min(
                    (current.0 as f64 + fraction) as u64,
                    max.0,
                )))
            }
            Change {
                direction: ChangeDirection::Decrease,
                magnitude: Magnitude::Percentage(percentage),
            } => {
                let fraction = (current.0 as f64 / 100.0) * percentage;
                Ok(Brightness(std::cmp::max(
                    (current.0 as f64 - fraction) as u64,
                    1,
                )))
            }
            Change {
                direction: ChangeDirection::Absolute,
                magnitude: Magnitude::Percentage(percentage),
            } => Ok(Brightness(((percentage / 100.0) * max.0 as f64) as u64)),
            Change {
                direction: ChangeDirection::Increase,
                magnitude: Magnitude::Absolute(value),
            } => Ok(Brightness(std::cmp::min(current.0 + value, max.0))),
            Change {
                direction: ChangeDirection::Decrease,
                magnitude: Magnitude::Absolute(value),
            } => Ok(Brightness(std::cmp::max(current.0 - value, 1))),
            Change {
                direction: ChangeDirection::Absolute,
                magnitude: Magnitude::Absolute(value),
            } => Ok(Brightness(std::cmp::max(value, max.0))),
        }
    }

    pub fn find_file(filename: &str) -> Result<PathBuf, DimError> {
        let glob_path = format!("{SYS_BACKLIGHT_PREFIX}/*/{filename}");
        let path = glob(&glob_path)?.next().ok_or(DimError::GlobError)??;
        if !path.is_file() {
            return Err(DimError::FileNotFound(path.to_string_lossy().to_string()));
        }
        Ok(path)
    }

    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Brightness, DimError> {
        let path = path.as_ref();
        let res = std::fs::read_to_string(path)?.trim().parse()?;
        Ok(res)
    }
}
