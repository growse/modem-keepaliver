use anyhow::Result;

use std::collections::HashMap;

use log::{debug, info, warn};

use num_traits::FromPrimitive;
use zbus::Connection;

use zbus::export::zvariant;
use zbus::export::zvariant::{OwnedObjectPath, OwnedValue};

use crate::dbus_proxies::MMModemState;

pub static MODEM_PATH: &str = "/org/freedesktop/ModemManager1/Modem/0";

pub async fn check_modem_state(connection: &Connection) -> Result<Option<MMModemState>> {
    let proxy = crate::dbus_proxies::SimpleProxy::new_for_path(&connection, MODEM_PATH)?;
    debug!("Fetching modem status");
    let status: HashMap<String, zvariant::OwnedValue> = proxy.get_status()?;
    let modem_state = modem_properties_to_status(&status);
    debug!("Modem state is: {:?}", modem_state);
    return Ok(modem_state);
}

pub async fn check_modem_state_and_maybe_reconnect(connection: &Connection) -> Result<()> {
    let modem_state = check_modem_state(connection).await?;
    match modem_state {
        Some(MMModemState::Registered) => {
            info!("Modem registered. Let's try and reconnect");
            simple_connect(&connection).await?;
        }
        Some(other) => {
            debug!("Modem is in state {:?}. Let's not bother it", other)
        }
        _ => {
            warn!("Modem doesn't appear to be giving us a state. Erk.");
        }
    }
    Ok(())
}

fn modem_properties_to_status(status: &HashMap<String, OwnedValue>) -> Option<MMModemState> {
    status
        .get("state")
        .and_then(|a| a.downcast_ref::<u32>())
        .and_then(|val| MMModemState::from_u32(*val))
}

async fn simple_connect(connection: &Connection) -> Result<()> {
    let proxy = crate::dbus_proxies::SimpleProxy::new_for_path(connection, MODEM_PATH)?;
    let connect_parameters = HashMap::from([
        ("apn", "giffgaff.com"),
        ("user", "gg"),
        ("password", "p"),
        ("allowed-auth", "pap"),
    ]);

    debug!(
        "Connecting to modem with parameters {:?}",
        connect_parameters
    );
    let bearer_path: OwnedObjectPath = proxy.connect(
        connect_parameters
            .iter()
            .map(|k| (*k.0, zvariant::Value::from(k.1)))
            .collect(),
    )?;
    info!(
        "Successfully connected modem. Bearer is {}",
        bearer_path.as_str()
    );

    Ok(())
}
