use glob::glob;
use thiserror::Error;

use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
pub const SYS_BACKLIGHT_PREFIX: &str = "/sys/class/backlight";

mod brightness;
pub use brightness::*;
mod change;

#[derive(Error, Debug)]
pub enum DimError {
    #[error("Invalid percentage given by user")]
    InvalidPercentage,
    #[error("Failed to parse invalid Brightness")]
    InvalidBrightness(#[from] std::num::ParseIntError),
    #[error("Failed to read/write file: {0}")]
    IOError(#[from] std::io::Error),
    #[error("Failed to find brightness file: ")]
    GlobError,
    #[error("Failed to find brightness file: ")]
    PatternError(#[from] glob::PatternError),
}

impl From<glob::GlobError> for DimError {
    fn from(_: glob::GlobError) -> DimError {
        DimError::GlobError
    }
}

impl From<std::num::ParseFloatError> for DimError {
    fn from(_: std::num::ParseFloatError) -> DimError {
        DimError::InvalidPercentage
    }
}

pub fn find_file(filename: &str) -> Result<PathBuf, DimError> {
    let glob_path = format!("{SYS_BACKLIGHT_PREFIX}/*/{filename}");
    let path = glob(&glob_path)?.next().ok_or(DimError::GlobError)??;
    Ok(path)
}

pub fn set_brightness<F: Write>(mut f: F, brightness: Brightness) -> Result<(), DimError> {
    write!(f, "{}", brightness)?;
    Ok(())
}

pub fn save<P: AsRef<Path>>(state_file: P, brightness: Brightness) -> Result<(), DimError> {
    let mut output = File::create(&state_file)?;
    write!(output, "{}", brightness)?;
    Ok(())
}
