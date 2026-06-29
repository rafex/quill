#[derive(Debug, PartialEq, Eq)]
pub enum RepositoryError {
    Duplicate,
    Unknown(String),
}
