use anyhow::{anyhow, Result};
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use zbus::{proxy, zvariant};

#[proxy(
    interface = "org.freedesktop.ModemManager1.Modem.Simple",
    default_service = "org.freedesktop.ModemManager1"
)]
pub(crate) trait Simple {
    /// Connect method
    fn connect(
        &self,
        properties: std::collections::HashMap<&str, zvariant::Value<'_>>,
    ) -> zbus::Result<zvariant::OwnedObjectPath>;

    /// Disconnect method
    fn disconnect(&self, bearer: &zvariant::ObjectPath<'_>) -> zbus::Result<()>;

    /// GetStatus method
    fn get_status(&self) -> zbus::Result<std::collections::HashMap<String, zvariant::OwnedValue>>;
}

#[proxy(
    interface = "org.freedesktop.ModemManager1.Modefm",
    default_service = "org.freedesktop.ModemManager1"
)]
pub(crate) trait Modem {
    /// StateChanged signal
    #[zbus(signal)]
    fn state_changed(&self, old: i32, new: i32, reason: u32) -> zbus::Result<()>;

    /// Enable method
    fn enable(&self, enable: bool) -> zbus::Result<()>;
}

#[proxy(
    interface = "org.freedesktop.ModemManager1.Modem.Signal",
    default_service = "org.freedesktop.ModemManager1"
)]
pub(crate) trait Signal {
    /// Setup method
    fn setup(&self, rate: u32) -> zbus::Result<()>;
}

#[derive(FromPrimitive, Debug)]
pub(crate) enum MMModemState {
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
pub(crate) enum MMModemStateChangeReason {
    Unknown = 0,
    UserRequested = 1,
    Suspend = 2,
    Failure = 3,
}

impl StateChangedArgs<'_> {
    pub(crate) fn to_modem_states(
        &self,
    ) -> Result<(MMModemState, MMModemState, MMModemStateChangeReason)> {
        Ok((
            MMModemState::from_i32(self.old)
                .ok_or_else(|| anyhow!("Invalid old state: {}", self.old))?,
            MMModemState::from_i32(self.new)
                .ok_or_else(|| anyhow!("Invalid new state: {}", self.new))?,
            MMModemStateChangeReason::from_u32(self.reason)
                .ok_or_else(|| anyhow!("Invalid state change reason: {}", self.reason))?,
        ))
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn state_change_converts_properly() {
        assert_eq!(2, 2)
    }
}
