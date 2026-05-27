use crate::DimError;

mod sysfs;
pub use crate::dimmer::sysfs::Sysfs;

mod dbus;
#[cfg(feature = "dbus")]
pub use crate::dimmer::dbus::Dbus;

pub trait Dimmer {
    fn set(&mut self, brightness: crate::Brightness) -> Result<(), DimError>;
}
