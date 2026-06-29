pub trait EventPublisher: Send + Sync {
    fn publish(&self, topic: &str, payload: &str) -> Result<(), PublishError>;
}

#[derive(Debug, PartialEq, Eq)]
pub enum PublishError {
    Unknown(String),
}
