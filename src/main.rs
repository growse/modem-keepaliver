mod dbus_proxies;
mod modem_manager;

use clap::Parser;

use std::time::Duration;

use crate::modem_manager::{check_modem_state, check_modem_state_and_maybe_reconnect};

use log::{error, info, LevelFilter};
use simplelog::{ColorChoice, Config, TerminalMode};

use tokio::try_join;

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

    match try_join!(periodic_modem_status_checker_loop) {
        Err(e) => error!("Boo: {}", e),
        _ => {}
    }
    Ok(())
}

fn get_modem_status_checker_loop_task(interval: Duration) -> tokio::task::JoinHandle<()> {
    tokio::task::spawn(async move {
        let mut task_interval = tokio::time::interval(interval);
        info!("Checking modem status every {}s", interval.as_secs());

        let connection = zbus::Connection::new_system();
        if connection.is_ok() {
            // Capability test. Don't start the loop if we can't fetch status
            let state = check_modem_state(connection.as_ref().unwrap()).await;

            if state.is_ok() {
                loop {
                    task_interval.tick().await;
                    let result =
                        check_modem_state_and_maybe_reconnect(connection.as_ref().unwrap()).await;
                    match result {
                        Err(e) => {
                            error!("Failed to poll and reconnect modem: {}", e);
                            break;
                        }
                        _ => {}
                    }
                }
            } else {
                error!("Failed to call DBus method: {}", state.err().unwrap())
            }
        } else {
            error!("Failed to connect to DBus: {}", connection.err().unwrap())
        }
    })
}
