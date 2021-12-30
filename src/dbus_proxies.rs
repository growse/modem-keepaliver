use num_derive::FromPrimitive;

use zbus::dbus_proxy;
use zbus::export::zvariant;

#[dbus_proxy(
    interface = "org.freedesktop.ModemManager1.Modem.Simple",
    default_service = "org.freedesktop.ModemManager1"
)]
pub trait Simple {
    /// Connect method
    fn connect(
        &self,
        properties: std::collections::HashMap<&str, zvariant::Value>,
    ) -> zbus::Result<zvariant::OwnedObjectPath>;

    /// Disconnect method
    fn disconnect(&self, bearer: &zvariant::ObjectPath) -> zbus::Result<()>;

    /// GetStatus method
    fn get_status(&self) -> zbus::Result<std::collections::HashMap<String, zvariant::OwnedValue>>;
}

#[dbus_proxy(interface = "org.freedesktop.ModemManager1.Modem")]
pub trait Modem {
    /// StateChanged signal
    #[dbus_proxy(signal)]
    fn state_changed(&self, old: i32, new: i32, reason: u32) -> zbus::Result<()>;
}

#[derive(FromPrimitive, Debug)]
pub enum MMModemState {
    Failed = -1,
    Unknown = 0,
    Initializing = 1,
    Locked = 2,
    Disabled = 3,
    Disabling = 4,
    Enabling = 5,
    Enabled = 6,
    Searching = 7,
    Registered = 8,
    Disconnecting = 9,
    Connecting = 10,
    Connected = 11,
}

#[derive(FromPrimitive, Debug)]
pub enum MMModemStateChangeReason {
    Unknown = 0,
    UserRequested = 1,
    Suspend = 2,
    Failure = 3,
}
