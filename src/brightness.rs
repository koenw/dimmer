use crate::change::*;
use crate::DimError;
use std::ops::{Add, Deref, Sub};
use std::path::Path;
use std::str::FromStr;

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

impl Brightness {
    pub fn new(value: u64) -> Brightness {
        Brightness(value)
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

    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Brightness, DimError> {
        let path = path.as_ref();
        let res = std::fs::read_to_string(path)?.trim().parse()?;
        Ok(res)
    }
}
