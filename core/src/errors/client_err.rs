use std::fmt::{self, Display};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MainTaskError {
    ErrReceiverChannelClosed,
}

impl MainTaskError {
    fn as_str(&self) -> &str {
        match self {
            MainTaskError::ErrReceiverChannelClosed => "ERR_RECEIVER_CHANNEL_CLOSED",
        }
    }
}

impl Display for MainTaskError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str(),)
    }
}
