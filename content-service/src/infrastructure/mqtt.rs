use std::time::Duration;

use rumqttc::{Client, Connection, MqttOptions};

pub fn connect(client_id: &str, host: &str, port: u16) -> (Client, Connection) {
    let mut options = MqttOptions::new(client_id, host, port);
    options.set_keep_alive(Duration::from_secs(30));
    Client::new(options, 10)
}
