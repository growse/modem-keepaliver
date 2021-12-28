mod modem_manager;

use env_logger::Env;

use crate::modem_manager::{check_modem_status_and_reconnect, dbus_message_handler_loop};
use log::info;
use std::time::Duration;
use tokio::{join, try_join};

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let periodic_modem_status_checker_loop = tokio::task::spawn(async {
        let mut interval = tokio::time::interval(Duration::from_secs(5));

        loop {
            interval.tick().await;
            check_modem_status_and_reconnect().await;
        }
    });
    try_join!(periodic_modem_status_checker_loop, dbus_message_handler_loop());
    Ok(())
}
