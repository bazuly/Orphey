use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("{0}")]
    Url(String),
    #[error("{0}")]
    Network(String),
    #[error("{0}")]
    Database(String),
}
