# modem-keepaliver

A little utility to keep a ModemManager modem connected. It listens for modem events from DBus, and when it detects a modem moving into a state of "Registered", sends a request to connect the modem. It can also periodically poll the status and reconnect if it detects a modem in "Registered".
