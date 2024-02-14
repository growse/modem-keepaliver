use anyhow::Result;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use log::{debug, info, warn};

use num_traits::FromPrimitive;
use zbus::{zvariant, Connection};

use zbus::zvariant::{OwnedObjectPath, OwnedValue};

use crate::dbus_proxies::{MMModemState, ModemProxy, SignalProxy, SimpleProxy, StateChangedStream};

static MODEM_PATH: &str = "/org/freedesktop/ModemManager1/Modem/0";

pub(crate) async fn check_modem_state_and_maybe_reconnect(
    connection: &Connection,
    bottleneck: Arc<Mutex<()>>,
) -> Result<()> {
    let modem_state = check_modem_state(connection).await?;
    match modem_state {
        Some(MMModemState::Registered) => {
            info!("Modem in state Registered. Let's try and reconnect");
            simple_connect(connection, bottleneck).await?;
        }
        Some(MMModemState::Disabled) => {
            info!("Modem is in state Disabled. Let's try and enable it");
            enable_modem(connection, bottleneck).await?;
        }
        Some(other) => {
            debug!("Modem is in state {other:?}. Let's not bother it")
        }
        _ => {
            warn!("Modem doesn't appear to be giving us a state. Erk.");
        }
    }
    Ok(())
}

async fn check_modem_state(connection: &Connection) -> Result<Option<MMModemState>> {
    let proxy = SimpleProxy::builder(connection)
        .path(MODEM_PATH)?
        .build()
        .await?;
    debug!("Fetching modem status");
    let status = proxy.get_status().await?;
    let modem_state = modem_properties_to_status(&status);
    debug!("Modem state is: {modem_state:?}");
    Ok(modem_state)
}

fn modem_properties_to_status(status: &HashMap<String, OwnedValue>) -> Option<MMModemState> {
    status
        .get("state")
        .map(|a| a.downcast_ref::<u32>().expect("State was not a u32"))
        .and_then(MMModemState::from_u32)
}

pub(crate) async fn simple_connect(
    connection: &Connection,
    bottleneck: Arc<Mutex<()>>,
) -> Result<()> {
    let guard = bottleneck.try_lock();
    if guard.is_ok() {
        debug!("Took connection guard");
        let proxy = SimpleProxy::builder(connection)
            .path(MODEM_PATH)?
            .build()
            .await?;
        let connect_parameters = HashMap::from([
            ("apn", "giffgaff.com"),
            ("user", "gg"),
            ("password", "p"),
            ("allowed-auth", "pap"),
        ]);

        debug!("Connecting to modem with parameters {connect_parameters:?}");
        let bearer_path: OwnedObjectPath = proxy
            .connect(
                connect_parameters
                    .iter()
                    .map(|k| (*k.0, zvariant::Value::from(k.1)))
                    .collect(),
            )
            .await?;
        info!(
            "Successfully connected modem. Bearer is {}",
            bearer_path.as_str()
        );
        enable_stats(connection).await?;
    } else {
        warn!("Unable to take guard: {:?}", guard.err().unwrap());
    }
    Ok(())
}

pub(crate) async fn get_state_change_stream<'a>(
    connection: &Connection,
) -> Result<StateChangedStream<'a>> {
    let proxy = ModemProxy::builder(connection)
        .path(MODEM_PATH)?
        .build()
        .await?;
    Ok(proxy.receive_state_changed().await?)
}

pub(crate) async fn enable_modem(
    connection: &Connection,
    bottleneck: Arc<Mutex<()>>,
) -> Result<()> {
    let guard = bottleneck.try_lock();
    if guard.is_ok() {
        let proxy = ModemProxy::builder(connection)
            .path(MODEM_PATH)?
            .build()
            .await?;
        Ok(proxy.enable(true).await?)
    } else {
        warn!("Unable to take guard: {:?}", guard.err().unwrap());
        Ok(())
    }
}

async fn enable_stats(connection: &Connection) -> Result<()> {
    info!("Enabling modem signal stats");
    let proxy = SignalProxy::builder(connection)
        .path(MODEM_PATH)?
        .build()
        .await?;
    Ok(proxy.setup(5).await?)
}
