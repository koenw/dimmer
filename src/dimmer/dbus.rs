#![cfg(feature = "dbus")]

use crate::{DimError, Dimmer};

pub struct Dbus {
    connection: dbus::blocking::Connection,
}

impl Dbus {
    pub fn new() -> Result<Self, DimError> {
        let connection = dbus::blocking::Connection::new_system()?;
        Ok(Self { connection })
    }
}

impl Dimmer for Dbus {
    fn set(&mut self, brightness: crate::Brightness) -> Result<(), DimError> {
        let msg = dbus::message::Message::call_with_args(
            "org.freedesktop.login1",
            "/org/freedesktop/login1/session/auto",
            "org.freedesktop.login1.Session",
            "SetBrightness",
            ("backlight", "intel_backlight", *brightness),
        );

        use dbus::blocking::BlockingSender;
        let _ = self
            .connection
            .send_with_reply_and_block(msg, std::time::Duration::from_millis(100))?;

        Ok(())
    }
}
