use std::sync::{Arc, Mutex};
use std::time::Duration;

use anyhow::Result;
use clap::Parser;
use log::{debug, error, info, LevelFilter};
use simplelog::{ColorChoice, Config, TerminalMode};
use tokio::try_join;
use zbus::export::ordered_stream::OrderedStreamExt;
use zbus::Connection;

use crate::dbus_proxies::{MMModemState, StateChangedStream};
use crate::modem_manager::{
    check_modem_state_and_maybe_reconnect, enable_modem, get_state_change_stream, simple_connect,
};

mod dbus_proxies;
mod modem_manager;

#[derive(Parser, Debug)]
#[clap(about, version, author)]
struct Args {
    /// Interval between periodic checks (in seconds)
    #[clap(long, default_value_t = 60)]
    check_interval: u64,

    /// Enabling dry-run means no requests will actually be sent to the ModemManager
    #[clap(long)]
    dry_run: bool,

    /// Enable debug logging
    #[clap(long)]
    debug: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    simplelog::TermLogger::init(
        if args.debug {
            LevelFilter::Debug
        } else {
            LevelFilter::Info
        },
        Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Always,
    )?;
    debug!("Debug logging enabled");

    let periodic_check_interval = Duration::from_secs(args.check_interval);

    let bottleneck = Arc::new(Mutex::new(true));

    try_join!(
        get_modem_status_checker_loop(periodic_check_interval, bottleneck.clone()),
        get_dbus_signal_listener(bottleneck.clone())
    )
    .map(|_| {})
    .map_err(|e| {
        error!("Fatal error: {e}");
        e
    })
}

async fn get_dbus_signal_listener(bottleneck: Arc<Mutex<bool>>) -> Result<()> {
    let connection = Connection::system().await?;
    let mut stream: StateChangedStream = get_state_change_stream(&connection).await?;

    info!("Listening for DBus Modem state_change signals");

    while let Some(signal) = stream.next().await {
        let state_change_event = signal
            .args()
            .map_err(anyhow::Error::from)
            .and_then(|a: dbus_proxies::StateChangedArgs| a.to_modem_states())?;
        debug!(
            "Modem state change detected. From {:?} to {:?} because of {:?}",
            state_change_event.0, state_change_event.1, state_change_event.2
        );
        match state_change_event {
            (old, MMModemState::Registered, _) => {
                info!("Modem changed from {old:?} to registered. Attempting connect.");
                simple_connect(&connection, bottleneck.clone()).await?;
            }
            (old, MMModemState::Disabled, reason) => {
                info!("Modem changed from {old:?} to disabled because of {reason:?}. Attempting to re-enable.");
                enable_modem(&connection, bottleneck.clone()).await?;
            }
            (_, MMModemState::Connected, _) => info!("Modem state changed to connected!"),
            state_change => debug!("Uninteresting state change: {state_change:?}"),
        }
    }
    Ok(())
}

async fn get_modem_status_checker_loop(
    interval: Duration,
    bottleneck: Arc<Mutex<bool>>,
) -> Result<()> {
    let mut task_interval = tokio::time::interval(interval);
    info!("Checking modem status every {}s", interval.as_secs());

    let connection = Connection::system().await?;

    let err = loop {
        task_interval.tick().await;
        let result = check_modem_state_and_maybe_reconnect(&connection, bottleneck.clone()).await;
        match result {
            Err(e) => {
                break e;
            }
            _ => {}
        }
    };
    Err::<(), anyhow::Error>(err)
}
