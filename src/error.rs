use std::{fmt::Display, string::FromUtf8Error};

use serde::Deserialize;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Reqwest(reqwest::Error),
    SerdeJson(serde_json::Error),
    IO(std::io::Error),
    FromUTF8(FromUtf8Error),
    Discord(DiscordError),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Reqwest(e) => write!(f, "reqwest error: {e}"),
            Error::SerdeJson(e) => write!(f, "serde_json error: {e}"),
            Error::IO(e) => write!(f, "std::io error: {e}"),
            Error::FromUTF8(e) => write!(f, "FromUTF8 error: {e}"),
            Error::Discord(e) => write!(f, "discord error: {e}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Reqwest(e) => Some(e),
            Error::SerdeJson(e) => Some(e),
            Error::IO(e) => Some(e),
            Error::FromUTF8(e) => Some(e),
            Error::Discord(e) => Some(e),
        }
    }
}

impl From<reqwest::Error> for Error {
    fn from(value: reqwest::Error) -> Self {
        Self::Reqwest(value)
    }
}
impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Self::SerdeJson(value)
    }
}
impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::IO(value)
    }
}
impl From<FromUtf8Error> for Error {
    fn from(value: FromUtf8Error) -> Self {
        Self::FromUTF8(value)
    }
}
impl From<DiscordError> for Error {
    fn from(value: DiscordError) -> Self {
        Self::Discord(value)
    }
}

#[derive(Deserialize, Debug)]
pub struct DiscordError {
    pub message: String,
    pub retry_after: f64,
    pub global: bool,
}

impl Display for DiscordError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for DiscordError {}
