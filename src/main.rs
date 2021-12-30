mod dbus_proxies;
mod modem_manager;

use clap::Parser;

use std::time::Duration;

use crate::dbus_proxies::{MMModemState, MMModemStateChangeReason};
use crate::modem_manager::{check_modem_state, check_modem_state_and_maybe_reconnect, MODEM_PATH};
use log::{debug, error, info, LevelFilter};
use num_traits::FromPrimitive;
use simplelog::{ColorChoice, Config, TerminalMode};
use tokio::task::JoinHandle;
use tokio::try_join;

use zbus::MessageError;

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
async fn main() -> anyhow::Result<()> {
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

    let periodic_check_interval = Duration::from_secs(args.check_interval);
    let periodic_modem_status_checker_loop =
        get_modem_status_checker_loop_task(periodic_check_interval);

    let dbus_signal_listener_task = get_dbus_signal_listener_and_doer_thing();

    match try_join!(
        periodic_modem_status_checker_loop,
        dbus_signal_listener_task
    ) {
        Err(e) => error!("Boo: {}", e),
        _ => {}
    }
    Ok(())
}

fn get_dbus_signal_listener_and_doer_thing() -> JoinHandle<anyhow::Result<()>> {
    tokio::task::spawn(async {
        let connection = zbus::Connection::new_system()?;
        let proxy = crate::dbus_proxies::ModemProxy::new_for_path(&connection, MODEM_PATH)?;
        proxy.connect_state_changed(move |old, new, reason| {
            match (
                MMModemState::from_i32(old),
                MMModemState::from_i32(new),
                MMModemStateChangeReason::from_u32(reason),
            ) {
                (Some(o), Some(n), Some(r)) => {
                    info!("State changed {:?} to {:?} because {:?}", o, n, r);
                    Ok(())
                }
                _ => Err(zbus::Error::from(MessageError::InvalidField)),
            }
        })?;
        debug!("Listening for DBus state_change signals");
        loop {
            let result = proxy.next_signal();
            if result.is_err() {
                break;
            }
        }
        Ok(())
    })
}

fn get_modem_status_checker_loop_task(interval: Duration) -> JoinHandle<anyhow::Result<()>> {
    tokio::task::spawn(async move {
        let mut task_interval = tokio::time::interval(interval);
        info!("Checking modem status every {}s", interval.as_secs());

        let connection = zbus::Connection::new_system()?;

        // Capability test. Don't start the loop if we can't fetch status
        check_modem_state(&connection).await?;

        loop {
            task_interval.tick().await;
            let result = check_modem_state_and_maybe_reconnect(&connection).await;
            match result {
                Err(e) => {
                    error!("Failed to poll and reconnect modem: {}", e);
                    break;
                }
                _ => {}
            }
        }
        Ok(())
    })
}
