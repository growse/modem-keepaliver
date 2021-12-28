use anyhow::Result;
use dbus::blocking::Connection;
use dbus::channel::MatchingReceiver;
use dbus::message::MatchRule;
use dbus::Message;

use log::{debug, info};
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

use std::time::Duration;

#[derive(FromPrimitive, Debug)]
enum MMModemState {
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
enum MMModemStateChangeReason {
    Unknown = 0,
    UserRequested = 1,
    Suspend = 2,
    Failure = 3,
}

pub async fn check_modem_status_and_reconnect() -> Result<()> {
    info!("Ping");
    Ok(())
}

fn handle_message(msg: &Message) {
    debug!("Received DBus message: {:?}", msg);
    // msg.path().expect("No path in dbus message");
    // msg.member().map(|m| {
    //     if m.into_cstring()
    //         .into_string()
    //         .expect("Member is not a valid string")
    //         .eq("StateChanged")
    //     {
    //         let (from_raw, to_raw, reason_raw) = msg.get3::<i32, i32, u32>();
    //         let from: Option<MMModemState> = from_raw.and_then(|x| FromPrimitive::from_i32(x));
    //         let to: Option<MMModemState> = to_raw.and_then(|x| FromPrimitive::from_i32(x));
    //         let reason: Option<MMModemStateChangeReason> =
    //             reason_raw.and_then(|x| FromPrimitive::from_u32(x));
    //
    //         match (from, to) {
    //             (Some(MMModemState::Disconnecting), Some(MMModemState::Registered)) => {
    //                 info!("Modem has disconnected back into registering state");
    //                 request_reconnect()
    //             }
    //             (f, t) => {
    //                 info!(
    //                     "State changed from {:?} to {:?} because of {:?}",
    //                     f, t, reason
    //                 );
    //             }
    //         }
    //     }
    // });
}

fn request_reconnect() {
    info!("Requesting reconnect of modem")
    // let result = proxy.method_call(
    //     "org.freedesktop.ModemManager1.Modem.Simple",
    //     "Connect",
    //     ("apn=giffgaff",),
    // );
    // result.expect("Error sending connect message")
}

pub async fn dbus_message_handler_loop() -> Result<()> {
    info!("FNARR");
    let connection = Connection::new_system().expect("Failed to connect to D-Bus");
    debug!("Connected to DBus");

    let rule = MatchRule::new();

    let proxy = connection.with_proxy(
        "org.freedesktop.DBus",
        "/org/freedesktop/ModemManager1/Modem",
        Duration::from_millis(1000),
    );
    debug!(
        "DBus proxy created destination={:} path={:}",
        proxy.destination, proxy.path
    );

    proxy
        .method_call(
            "org.freedesktop.DBus.Monitoring",
            "BecomeMonitor",
            (vec![rule.match_str()], 0u32),
        )
        .and_then(|_: ()| {
            debug!("Starting receive");
            Ok(connection.start_receive(
                rule.static_clone(),
                Box::new(|msg, _| {
                    handle_message(&msg);
                    true
                }),
            ))
        })
        .or_else::<dbus::Error, _>(|_| {
            debug!("Adding match");
            connection.add_match(rule, |_: (), _, msg| {
                handle_message(&msg);
                true
            })
        })
        .expect("Unable to subscribe to DBus");

    loop {
        connection
            .process(Duration::from_secs(1))
            .expect("DBus connection processing timed out");
    }
}
