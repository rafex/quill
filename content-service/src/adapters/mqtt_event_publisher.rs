use rumqttc::{Client, QoS};
use std::sync::Mutex;

use crate::ports::{EventPublisher, PublishError};

pub struct MqttEventPublisher {
    client: Mutex<Client>,
}

impl MqttEventPublisher {
    pub fn new(client: Client) -> Self {
        Self {
            client: Mutex::new(client),
        }
    }
}

impl EventPublisher for MqttEventPublisher {
    fn publish(&self, topic: &str, payload: &str) -> Result<(), PublishError> {
        self.client
            .lock()
            .unwrap()
            .publish(topic, QoS::AtLeastOnce, false, payload.as_bytes())
            .map_err(|e| PublishError::Unknown(e.to_string()))
    }
}
