use thiserror::Error;

mod brightness;
pub use brightness::*;
mod change;
pub mod dimmer;
pub use dimmer::Dimmer;

#[derive(Error, Debug)]
pub enum DimError {
    #[error("Invalid percentage given by user")]
    InvalidPercentage,
    #[error("Failed to parse invalid Brightness")]
    InvalidBrightness(#[from] std::num::ParseIntError),
    #[error("Failed to read/write file: {0}")]
    IOError(#[from] std::io::Error),
    #[error("Failed to find brightness file")]
    GlobError,
    #[error("Failed to find brightness file: {0}")]
    PatternError(#[from] glob::PatternError),
    #[error("Failed to send dbus message: {0}")]
    Dbus(#[from] dbus::Error),
    #[error("Failed to open file: {0}")]
    FileNotFound(String),
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
