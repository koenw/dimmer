use crate::{DimError, Dimmer};

pub struct Sysfs<W: std::io::Write> {
    f: W,
}

impl<W: std::io::Write> Sysfs<W> {
    pub fn with_file(f: W) -> Self {
        Self { f }
    }
}

impl<W: std::io::Write> Dimmer for Sysfs<W> {
    fn set(&mut self, brightness: crate::Brightness) -> Result<(), DimError> {
        write!(self.f, "{brightness}")?;
        Ok(())
    }
}
